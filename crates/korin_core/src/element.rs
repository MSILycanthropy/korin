use crate::primitives::{div::Div, text::Text};

pub enum Element<'a> {
    Div(Box<Div<'a>>),
    Text(Text<'a>),
}

impl<'a> Element<'a> {
    pub fn div(div: Div<'a>) -> Self {
        Self::Div(Box::new(div))
    }

    pub const fn text(text: Text<'a>) -> Self {
        Self::Text(text)
    }
}
