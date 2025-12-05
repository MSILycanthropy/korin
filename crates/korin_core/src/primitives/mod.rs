use ratatui::{buffer::Buffer, layout::Rect};

pub mod div;
pub mod text;

pub trait Primitive {
    fn render(&self, area: Rect, buf: &mut Buffer);

    fn taffy_style(&self) -> taffy::Style;

    fn is_focusable(&self) -> bool {
        false
    }
}

pub type AnyPrimitive<'a> = Box<dyn Primitive + 'a>;
