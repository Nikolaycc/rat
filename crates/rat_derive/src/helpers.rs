use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Type, TypePath};

pub(crate) fn map_primitive_to_zerocopy(ty: &Type) -> Option<Type> {
    let Type::Path(TypePath {
        path, qself: None, ..
    }) = ty
    else {
        return None;
    };

    let ident = path.get_ident()?;

    let replacement = match ident.to_string().as_str() {
        "u16" => "U16",
        "u32" => "U32",
        "u64" => "U64",
        "u128" => "U128",
        "i16" => "I16",
        "i32" => "I32",
        "i64" => "I64",
        "i128" => "I128",
        _ => return None,
    };

    let replacement_ident = syn::Ident::new(replacement, ident.span());

    Some(syn::parse_quote! {
        ::zerocopy::byteorder::network_endian::#replacement_ident
    })
}

pub(crate) fn rat_crate_path() -> TokenStream2 {
    if std::env::var("CARGO_BIN_NAME").is_ok() {
        return quote! { ::rat };
    }

    match crate_name("rat") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote! { ::#ident }
        }
        Err(_) => quote! { ::rat }, // fallback, shouldn't normally hit this
    }
}
