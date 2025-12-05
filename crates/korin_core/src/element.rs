use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::primitives::{AnyPrimitive, div::Div, text::Text};

pub struct Element<'a> {
    inner: AnyPrimitive<'a>,
}

impl<'a> Element<'a> {
    #[must_use]
    pub fn new(primitive: AnyPrimitive<'a>) -> Self {
        Self { inner: primitive }
    }

    #[must_use]
    pub fn div(div: Div<'a>) -> Self {
        Self::new(Box::new(div))
    }

    #[must_use]
    pub fn text(text: Text<'a>) -> Self {
        Self::new(Box::new(text))
    }

    #[must_use]
    pub fn to_inner(&self) -> &AnyPrimitive<'a> {
        &self.inner
    }
}

impl Widget for Element<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &Element<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.to_inner().render(area, buf);
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

impl<'a> From<&Element<'a>> for taffy::Style {
    fn from(value: &Element<'a>) -> Self {
        value.to_inner().taffy_style()
    }
}
