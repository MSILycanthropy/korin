use ratatui::{buffer::Buffer, crossterm::event::KeyEvent, layout::Rect, widgets::Widget};

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

    #[must_use]
    pub fn is_focusable(&self) -> bool {
        self.inner.is_focusable()
    }

    pub fn on_key(&mut self, event: KeyEvent) {
        self.inner.on_key(event);
    }

    pub fn on_focus(&mut self) {
        self.inner.on_focus();
    }

    pub fn on_blur(&mut self) {
        self.inner.on_blur();
    }
}

impl Widget for Element<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &Element<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.inner.render(area, buf);
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
