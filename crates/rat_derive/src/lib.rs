mod helpers;
mod packet_args;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

use crate::{
    helpers::rat_crate_path,
    packet_args::{PacketArgs, process_fields},
};

#[proc_macro_attribute]
pub fn packet(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as PacketArgs);

    let mut item_struct = parse_macro_input!(item as ItemStruct);

    let rat_path = rat_crate_path();

    let processed = match process_fields(&mut item_struct) {
        Ok(fields) => fields,

        Err(error) => {
            return error.to_compile_error().into();
        }
    };

    let struct_ident = &item_struct.ident;

    let layer = &args.layer;

    let selector = &args.selector;

    let next_field_access = match &processed.next {
        Some(info) if info.is_zerocopy_wrapped => {
            let field = &info.ident;

            quote! {
                Some(
                    self.#field.get()
                        as u32
                )
            }
        }

        Some(info) => {
            let field = &info.ident;

            quote! {
                Some(
                    self.#field
                        as u32
                )
            }
        }

        None => {
            quote! {
                None
            }
        }
    };

    let parent_type_id_body = if let Some(parent) = &args.parent {
        quote! {
            Some(
                ::std::any::TypeId::of::<#parent>()
            )
        }
    } else {
        quote! {
            None
        }
    };

    let label_width = processed
        .display
        .iter()
        .map(|field| field.label.value().chars().count() + 1)
        .max()
        .unwrap_or(0);

    let display_fields = processed.display.iter().map(|info| {
        let field = &info.ident;
        let label = &info.label;

        let value = if info.is_zerocopy_wrapped {
            quote! {
                let value = self.#field.get();
            }
        } else {
            quote! {
                let value = self.#field;
            }
        };

        if let Some(display_with) = &info.display_with {
            quote! {
                {
                    #value

                    ::std::write!(
                        f,
                        "    {:<width$} ",
                        ::std::concat!(#label, ":"),
                        width = #label_width,
                    )?;

                    #display_with(value, f)?;

                    ::std::writeln!(f)?;
                }
            }
        } else {
            quote! {
                {
                    #value

                    ::std::writeln!(
                        f,
                        "    {:<width$} {}",
                        ::std::concat!(#label, ":"),
                        value,
                        width = #label_width,
                    )?;
                }
            }
        }
    });

    let expanded = quote! {
        #[derive(
            Debug,
            PartialEq,
            Eq,
            ::zerocopy::TryFromBytes,
            ::zerocopy::KnownLayout,
            ::zerocopy::Immutable,
            ::zerocopy::Unaligned
        )]
        #[repr(C, packed)]
        #item_struct

        impl #rat_path::packet::Packet
            for #struct_ident
        {
            const LAYER:
                #rat_path::packet::Layer =
                #rat_path::packet::Layer::#layer;

            const SELECTOR: u32 =
                #selector as u32;

            fn parse(
                data: &[u8],
            ) -> ::std::result::Result<
                &Self,
                #rat_path::utils::ParseError,
            > {
                let size =
                    ::std::mem::size_of::<Self>();

                let data = data
                    .get(..size)
                    .ok_or(
                        #rat_path::utils::ParseError
                            ::InvalidValue
                    )?;

                <Self as ::zerocopy::TryFromBytes>
                    ::try_ref_from_bytes(data)
                    .map_err(|_| {
                        #rat_path::utils::ParseError
                            ::InvalidValue
                    })
            }

            fn parent_type_id(
            ) -> ::std::option::Option<
                ::std::any::TypeId
            > {
                #parent_type_id_body
            }

            fn next_selector(
                &self,
            ) -> ::std::option::Option<u32> {
                #next_field_access
            }
        }

        impl ::std::fmt::Display
            for #struct_ident
        {
            fn fmt(
                &self,
                f: &mut ::std::fmt::Formatter<'_>,
            ) -> ::std::fmt::Result {
                ::std::writeln!(
                    f,
                    "▼ {}",
                    ::std::stringify!(#struct_ident),
                )?;

                #(#display_fields)*

                Ok(())
            }
        }
    };

    expanded.into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
