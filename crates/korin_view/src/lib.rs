mod any;
mod container;
mod event;
mod into_view;
mod render;
mod text;

pub use container::Container;
pub use into_view::{IntoView, View};
pub use render::Render;
pub use text::Text;

#[must_use] 
pub fn container() -> Container {
    Container::new()
}

pub fn text(content: impl Into<String>) -> Text {
    Text::new(content)
}
