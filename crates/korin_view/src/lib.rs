mod container;
mod render;
mod style;
mod text;
mod view;

pub use container::Container;
pub use render::{Render, RenderContext};
pub use style::{AnyStyle, IntoAnyStyle};
pub use text::Text;
pub use view::{AnyState, AnyView, IntoAnyView};

#[must_use]
pub fn container<Ctx: RenderContext + Clone>() -> Container<Ctx> {
    Container::new()
}

pub fn text(content: impl Into<String>) -> Text {
    Text::new(content)
}
