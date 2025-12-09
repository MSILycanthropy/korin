mod container;
mod event;
mod render;
mod style;
mod text;
mod view;

pub use container::Container;
pub use event::{EventHandler, FocusHandler};
pub use render::{Render, RenderContext};
pub use style::{AnyStyle, IntoAnyStyle};
pub use text::Text;
pub use view::{AnyView, IntoView};

#[must_use]
pub fn container<Ctx: RenderContext + Clone>() -> Container<Ctx> {
    Container::new()
}

pub fn text(content: impl Into<String>) -> Text {
    Text::new(content)
}
