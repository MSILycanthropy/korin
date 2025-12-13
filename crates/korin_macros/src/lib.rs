use proc_macro::TokenStream;

mod component;
mod event;
mod utils;
mod view;

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    component::implmentation(item)
}

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    view::implmentation(input)
}

#[proc_macro_derive(Event, attributes(event))]
pub fn derive_event(input: TokenStream) -> TokenStream {
    event::implementation(input)
}
