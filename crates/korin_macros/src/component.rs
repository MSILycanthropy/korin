use convert_case::ccase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{FnArg, Ident, ItemFn, Pat, PatType, ReturnType, Type, Visibility};

use crate::utils::{extract_option_inner, is_option_type};

pub fn implmentation(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ItemFn);

    match generate_component(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_component(input: &ItemFn) -> syn::Result<TokenStream2> {
    let ret_type = validate_return_type(input)?;
    let props = extract_props(input)?;
    let names = generate_names(&input.sig.ident);

    let props_struct = generate_props_struct(&input.vis, &names, &props);
    let builder_struct = generate_builder_struct(&input.vis, &names, &props);
    let component_struct = generate_component_struct(&input.vis, &names, &props);
    let impl_fn = generate_impl_fn(&names, input, ret_type);

    Ok(quote! {
        #props_struct
        #builder_struct
        #component_struct
        #impl_fn
    })
}

struct ComponentNames {
    struct_name: Ident,
    props_ident: Ident,
    builder_ident: Ident,
    impl_fn_ident: Ident,
}

fn generate_names(fn_name: &Ident) -> ComponentNames {
    let struct_name = ccase!(pascal, &fn_name.to_string());

    ComponentNames {
        struct_name: format_ident!("{}", struct_name),
        props_ident: format_ident!("{}Props", struct_name),
        builder_ident: format_ident!("{}PropsBuilder", struct_name),
        impl_fn_ident: format_ident!("{}_impl", fn_name),
    }
}

struct Prop {
    name: Ident,
    ty: Box<Type>,
    is_optional: bool,
}

fn extract_props(input: &ItemFn) -> syn::Result<Vec<Prop>> {
    input
        .sig
        .inputs
        .iter()
        .map(|arg| {
            let pat_type = match arg {
                FnArg::Typed(pt) => pt,
                FnArg::Receiver(r) => {
                    return Err(syn::Error::new_spanned(r, "self not allowed in components"));
                }
            };

            extract_prop(pat_type)
        })
        .collect()
}

fn extract_prop(pat_type: &PatType) -> syn::Result<Prop> {
    let name = match pat_type.pat.as_ref() {
        Pat::Ident(pat_ident) => pat_ident.ident.clone(),
        other => return Err(syn::Error::new_spanned(other, "expected identifier")),
    };

    let ty = pat_type.ty.clone();
    let is_optional = is_option_type(&ty);

    Ok(Prop {
        name,
        ty,
        is_optional,
    })
}

fn validate_return_type(input: &ItemFn) -> syn::Result<&Type> {
    match &input.sig.output {
        ReturnType::Type(_, ty) => Ok(ty.as_ref()),
        ReturnType::Default => Err(syn::Error::new_spanned(
            &input.sig,
            "component must have a return type",
        )),
    }
}

fn generate_props_struct(vis: &Visibility, names: &ComponentNames, props: &[Prop]) -> TokenStream2 {
    let props_ident = &names.props_ident;
    let builder_ident = &names.builder_ident;

    let fields = props.iter().map(|p| {
        let name = &p.name;
        let ty = &p.ty;
        quote! { pub #name: #ty }
    });

    quote! {
        #vis struct #props_ident {
            #(#fields),*
        }

        impl #props_ident {
            pub fn builder() -> #builder_ident {
                #builder_ident::default()
            }
        }
    }
}

fn generate_builder_struct(
    vis: &Visibility,
    names: &ComponentNames,
    props: &[Prop],
) -> TokenStream2 {
    let props_ident = &names.props_ident;
    let builder_ident = &names.builder_ident;

    let builder_fields = props.iter().map(|p| {
        let name = &p.name;
        let ty = &p.ty;
        if p.is_optional {
            quote! { #name: #ty }
        } else {
            quote! { #name: Option<#ty> }
        }
    });

    let builder_defaults = props.iter().map(|p| {
        let name = &p.name;
        quote! { #name: None }
    });

    let builder_methods = props.iter().map(|p| {
        let name = &p.name;
        let ty = &p.ty;

        if p.is_optional {
            let inner_ty = extract_option_inner(ty);
            quote! {
                pub fn #name(mut self, value: #inner_ty) -> Self {
                    self.#name = Some(value);
                    self
                }
            }
        } else {
            quote! {
                pub fn #name(mut self, value: #ty) -> Self {
                    self.#name = Some(value);
                    self
                }
            }
        }
    });

    let build_fields = props.iter().map(|p| {
        let name = &p.name;
        if p.is_optional {
            quote! { #name: self.#name }
        } else {
            quote! { #name: self.#name.expect(concat!("required prop '", stringify!(#name), "' not set")) }
        }
    });

    quote! {
        #vis struct #builder_ident {
            #(#builder_fields),*
        }

        impl #builder_ident {
            pub fn new() -> Self {
                Self {
                    #(#builder_defaults),*
                }
            }

            #(#builder_methods)*

            pub fn build(self) -> #props_ident {
                #props_ident {
                    #(#build_fields),*
                }
            }
        }

        impl Default for #builder_ident {
            fn default() -> Self {
                Self::new()
            }
        }
    }
}

fn generate_component_struct(
    vis: &Visibility,
    names: &ComponentNames,
    props: &[Prop],
) -> TokenStream2 {
    let struct_name = &names.struct_name;
    let props_ident = &names.props_ident;
    let impl_fn_ident = &names.impl_fn_ident;

    let prop_names = props.iter().map(|p| &p.name);

    quote! {
        #vis struct #struct_name(pub #props_ident);

        impl korin_view::IntoView<korin_runtime::RuntimeContext> for #struct_name {
            fn into_view(self) -> korin_runtime::View {
                let result = #impl_fn_ident(#(self.0.#prop_names),*);

                korin_view::IntoView::into_view(result)
            }
        }
    }
}

fn generate_impl_fn(names: &ComponentNames, input: &ItemFn, ret_type: &Type) -> TokenStream2 {
    let impl_fn_ident = &names.impl_fn_ident;
    let inputs = &input.sig.inputs;
    let body = &input.block;

    quote! {
        fn #impl_fn_ident(#inputs) -> #ret_type #body
    }
}
