use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::primitives::{Primitive, div::Div, text::Text};

pub(crate) mod primitives;

pub enum View<'a> {
    Div(Box<Div<'a>>),
    Text(Text<'a>),
    Fragment(Vec<View<'a>>), // TODO: Fragment rendering
}

impl Widget for View<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self {
            View::Div(div) => div.to_widget().render(area, buf),
            View::Text(text) => text.to_widget().render(area, buf),
            View::Fragment(_) => {}
        }
    }
}
