use syn::{
    Expr, Ident, ItemStruct, Token,
    parse::{Parse, ParseStream},
};

use crate::helpers::map_primitive_to_zerocopy;

pub(crate) struct PacketArgs {
    pub layer: Ident,
    pub parent: Option<Ident>,
    pub selector: Expr,
}

impl Parse for PacketArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut layer = None;
        let mut parent = None;
        let mut selector = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "layer" => layer = Some(input.parse()?),
                "parent" => parent = Some(input.parse()?),
                "selector" => selector = Some(input.parse()?),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown packet() key `{other}`, expected `layer`, `parent`, or `selector`"
                        ),
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(PacketArgs {
            layer: layer.ok_or_else(|| syn::Error::new(input.span(), "missing `layer`"))?,
            parent,
            selector: selector
                .ok_or_else(|| syn::Error::new(input.span(), "missing `selector`"))?,
        })
    }
}

pub(crate) struct NextFieldInfo {
    pub ident: Ident,
    pub is_zerocopy_wrapped: bool,
}

pub(crate) fn process_fields(item: &mut ItemStruct) -> syn::Result<Option<NextFieldInfo>> {
    let mut next_field = None;

    if let syn::Fields::Named(fields) = &mut item.fields {
        for field in &mut fields.named {
            let mut is_next = false;
            field.attrs.retain(|attr| {
                if attr.path().is_ident("packet") {
                    is_next = attr.parse_args::<Ident>().is_ok_and(|id| id == "next");
                    false
                } else {
                    true
                }
            });

            let wrapped = map_primitive_to_zerocopy(&field.ty).map(|new_ty| {
                field.ty = new_ty;
            });

            if is_next {
                if next_field.is_some() {
                    return Err(syn::Error::new_spanned(
                        field,
                        "only one field can be marked #[packet(next)]",
                    ));
                }
                next_field = Some(NextFieldInfo {
                    ident: field.ident.clone().unwrap(),
                    is_zerocopy_wrapped: wrapped.is_some(),
                });
            }
        }
    }

    Ok(next_field)
}
