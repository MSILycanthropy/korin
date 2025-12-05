use ratatui::{buffer::Buffer, crossterm::event::KeyEvent, layout::Rect};

pub mod div;
pub mod text;

pub trait Primitive {
    fn render(&self, area: Rect, buf: &mut Buffer);

    fn taffy_style(&self) -> taffy::Style;

    fn is_focusable(&self) -> bool {
        false
    }

    fn on_key(&mut self, _event: KeyEvent) {}

    fn on_focus(&mut self) {}

    fn on_blur(&mut self) {}
}

pub type AnyPrimitive<'a> = Box<dyn Primitive + 'a>;
