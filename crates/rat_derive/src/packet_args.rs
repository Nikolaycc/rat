use syn::{
    Expr, Ident, ItemStruct, LitStr, Path, Token,
    parse::{Parse, ParseStream},
};

use crate::helpers::{map_primitive_to_zerocopy, normilze_field};

pub(crate) struct PacketArgs {
    pub layer: Ident,
    pub parent: Option<syn::Path>,
    pub selector: Expr,
    pub header_len: Option<Expr>,
    pub packet_len: Option<Expr>,
}

impl Parse for PacketArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut layer = None;
        let mut parent = None;
        let mut selector = None;
        let mut header_len = None;
        let mut packet_len = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "layer" => {
                    layer = Some(input.parse()?);
                }

                "parent" => {
                    parent = Some(input.parse()?);
                }

                "selector" => {
                    selector = Some(input.parse()?);
                }

                "header_len" => {
                    header_len = Some(input.parse()?);
                }

                "packet_len" => {
                    packet_len = Some(input.parse()?);
                }

                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown packet() key `{other}`, expected \
                             `layer`, `parent`, `selector`, `header_len`, or `packet_len`"
                        ),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            layer: layer.ok_or_else(|| syn::Error::new(input.span(), "missing `layer`"))?,
            parent,
            selector: selector
                .ok_or_else(|| syn::Error::new(input.span(), "missing `selector`"))?,
            header_len,
            packet_len,
        })
    }
}

pub(crate) struct FieldInfo {
    pub ident: Ident,
    pub is_zerocopy_wrapped: bool,
}

pub(crate) struct DisplayFieldInfo {
    pub ident: Ident,
    pub label: LitStr,
    pub display_with: Option<Path>,
    pub is_zerocopy_wrapped: bool,
}

pub(crate) struct ProcessedFields {
    pub next: Option<FieldInfo>,
    pub display: Vec<DisplayFieldInfo>,
}

#[derive(Default)]
struct PacketFieldArgs {
    next: bool,
    label: Option<LitStr>,
    display_with: Option<Path>,
}

pub(crate) fn process_fields(item: &mut ItemStruct) -> syn::Result<ProcessedFields> {
    let mut next_field = None;
    let mut display_fields = Vec::new();

    let syn::Fields::Named(fields) = &mut item.fields else {
        return Err(syn::Error::new_spanned(
            &item.fields,
            "`#[packet]` requires named fields",
        ));
    };

    for field in &mut fields.named {
        let ident = field
            .ident
            .clone()
            .ok_or_else(|| syn::Error::new_spanned(&*field, "packet fields must be named"))?;

        let mut packet_args = PacketFieldArgs::default();

        let attrs = std::mem::take(&mut field.attrs);

        for attr in attrs {
            if !attr.path().is_ident("packet") {
                field.attrs.push(attr);
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("next") {
                    if packet_args.next {
                        return Err(meta.error("duplicate `next`"));
                    }

                    packet_args.next = true;
                    return Ok(());
                }

                if meta.path.is_ident("label") {
                    if packet_args.label.is_some() {
                        return Err(meta.error("duplicate `label`"));
                    }

                    packet_args.label = Some(meta.value()?.parse()?);

                    return Ok(());
                }

                if meta.path.is_ident("display_with") {
                    if packet_args.display_with.is_some() {
                        return Err(meta.error("duplicate `display_with`"));
                    }

                    packet_args.display_with = Some(meta.value()?.parse()?);

                    return Ok(());
                }

                Err(meta.error("expected `next`, `label = \"...\"`, or `display_with = path`"))
            })?;
        }

        if packet_args.label.is_none() {
            packet_args.label = Some(LitStr::new(
                &normilze_field(&ident.to_string()),
                ident.span(),
            ));
        }

        let is_zerocopy_wrapped = map_primitive_to_zerocopy(&field.ty)
            .map(|new_ty| {
                field.ty = new_ty;
            })
            .is_some();

        if packet_args.next {
            if next_field.is_some() {
                return Err(syn::Error::new_spanned(
                    &*field,
                    "only one field can be marked #[packet(next)]",
                ));
            }

            next_field = Some(FieldInfo {
                ident: ident.clone(),
                is_zerocopy_wrapped,
            });
        }

        if let Some(label) = packet_args.label {
            display_fields.push(DisplayFieldInfo {
                ident,
                label,
                display_with: packet_args.display_with,
                is_zerocopy_wrapped,
            });
        } else if packet_args.display_with.is_some() {
            return Err(syn::Error::new_spanned(
                field,
                "`display_with` requires `label`",
            ));
        }
    }

    Ok(ProcessedFields {
        next: next_field,
        display: display_fields,
    })
}
