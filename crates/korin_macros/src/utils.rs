use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Type;

pub fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "Option";
    }

    false
}

pub fn is_string_type_ts(ty: &TokenStream2) -> bool {
    let s = ty.to_string();
    s == "String" || s == "std :: string :: String" || s == "alloc :: string :: String"
}

pub fn is_string_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
    {
        return segment.ident == "String";
    }
    false
}

pub fn extract_option_inner(ty: &Type) -> TokenStream2 {
    if let Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return quote! { #inner };
    }
    quote! { #ty }
}
