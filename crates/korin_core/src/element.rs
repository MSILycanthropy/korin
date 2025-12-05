use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::primitives::{Primitive, div::Div, text::Text};

pub enum Element<'a> {
    Div(Box<Div<'a>>),
    Text(Text<'a>),
}

impl<'a> Element<'a> {
    #[must_use]
    pub fn div(div: Div<'a>) -> Self {
        Self::Div(Box::new(div))
    }

    #[must_use]
    pub const fn text(text: Text<'a>) -> Self {
        Self::Text(text)
    }
}

impl Widget for Element<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &Element<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Element::Div(div) => div.to_widget().render(area, buf),
            Element::Text(text) => text.to_widget().render(area, buf),
        }
    }
}

impl<'a> From<Div<'a>> for Element<'a> {
    fn from(value: Div<'a>) -> Self {
        Self::div(value)
    }
}

impl<'a> From<Text<'a>> for Element<'a> {
    fn from(value: Text<'a>) -> Self {
        Self::text(value)
    }
}
