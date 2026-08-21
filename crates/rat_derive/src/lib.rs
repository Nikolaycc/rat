mod helpers;
mod packet_args;

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, parse_macro_input};

use crate::helpers::rat_crate_path;
use crate::packet_args::{PacketArgs, process_fields};

#[proc_macro_attribute]
pub fn packet(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as PacketArgs);
    let mut item_struct = parse_macro_input!(item as ItemStruct);

    let rat_path = rat_crate_path();

    let next_field = match process_fields(&mut item_struct) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
    };

    let next_field_access = match &next_field {
        Some(info) if info.is_zerocopy_wrapped => {
            let f = &info.ident;
            quote! { Some(self.#f.get() as u32) }
        }
        Some(info) => {
            let f = &info.ident;
            quote! { Some(self.#f as u32) }
        }
        None => quote! { None },
    };

    let struct_ident = &item_struct.ident;
    let layer = &args.layer;
    let selector = &args.selector;

    let parent_type_id_body = if let Some(parent) = &args.parent {
        quote! { Some(::std::any::TypeId::of::<#parent>()) }
    } else {
        quote! { None }
    };

    let expanded = quote! {
        #[derive(
            Debug, PartialEq, Eq,
            ::zerocopy::TryFromBytes,
            ::zerocopy::KnownLayout,
            ::zerocopy::Immutable,
            ::zerocopy::Unaligned
        )]
        #[repr(C, packed)]
        #item_struct

        impl #rat_path::packets::Packet for #struct_ident {
            const LAYER: #rat_path::packets::Layer = #rat_path::packets::Layer::OSI(#rat_path::packets::OSILayer::#layer);
            const SELECTOR: u32 = #selector as u32;

            fn parse(data: &[u8]) -> ::std::result::Result<&Self, #rat_path::utils::ParseError> {
                <Self as ::zerocopy::TryFromBytes>::try_ref_from_bytes(
                    &data[..size_of::<#struct_ident>()]
                )
                .map_err(|_| #rat_path::utils::ParseError::InvalidValue)
            }

            fn parent_type_id() -> Option<::std::any::TypeId> {
                #parent_type_id_body
            }

            fn next_selector(&self) -> ::std::option::Option<u32> {
                #next_field_access
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
