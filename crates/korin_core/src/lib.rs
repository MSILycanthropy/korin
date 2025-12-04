use korin_layout::Layout;
use ratatui::{crossterm::event::KeyEvent, style::Style};

pub enum TextContent {
    Static(String),
    Dynamic(Box<dyn Fn() -> String>),
}

pub enum View {
    Div {
        layout: Box<Layout>,
        style: Style,
        children: Vec<View>,
        focusable: bool,
        on_key: Option<Box<dyn Fn(KeyEvent)>>,
        on_focus: Option<Box<dyn Fn()>>,
        on_blur: Option<Box<dyn Fn()>>,
    },
    Text {
        content: TextContent,
        style: Style,
    },
    Fragment(Vec<View>),
}
