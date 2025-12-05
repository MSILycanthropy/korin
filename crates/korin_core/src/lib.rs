use crate::primitives::{div::Div, text::Text};

pub(crate) mod border;
pub(crate) mod primitives;

pub enum View<'a> {
    Div(Box<Div<'a>>),
    Text(Text),
    Fragment(Vec<View<'a>>),
}

pub enum TextContent {
    Static(String),
    Dynamic(Box<dyn Fn() -> String>),
}
