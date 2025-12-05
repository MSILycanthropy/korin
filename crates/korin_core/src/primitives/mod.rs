use ratatui::widgets::Widget;

pub mod div;
pub mod text;

pub trait Primitive<T: Widget> {
    fn to_widget(&self) -> T;
}
