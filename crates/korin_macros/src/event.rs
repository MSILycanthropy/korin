use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn implementation(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let mut bubbles = true;
    let mut krate = quote! { korin_event };

    for attr in &input.attrs {
        if attr.path().is_ident("event") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("bubbles") {
                    let value: syn::LitBool = meta.value()?.parse()?;
                    bubbles = value.value();
                } else if meta.path.is_ident("crate") {
                    let value: syn::Path = meta.value()?.parse()?;
                    krate = quote! { #value };
                }
                Ok(())
            })
            .ok();
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let bubbles_impl = if bubbles {
        quote! {}
    } else {
        quote! {
            fn bubbles() -> bool { false }
        }
    };

    quote! {
        impl #impl_generics #krate::Event for #name #ty_generics #where_clause {
            #bubbles_impl
        }
    }
    .into()
}
