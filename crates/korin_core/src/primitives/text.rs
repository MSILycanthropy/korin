use ratatui_core::{buffer::Buffer, layout::Rect, style::Style, widgets::Widget};

use crate::TextContent;

pub struct Text {
    pub content: TextContent,
    pub style: Style,
}

impl Widget for Text {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Widget::render(&self, area, buf);
    }
}

impl Widget for &Text {
    fn render(self, _area: Rect, _buf: &mut Buffer)
    where
        Self: Sized,
    {
    }
}
