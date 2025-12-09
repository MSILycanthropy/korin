use proc_macro::TokenStream;

mod component;
mod utils;

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    component::implmentation(item)
}
