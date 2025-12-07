use korin_layout::Layout;
use korin_style::Style;
use korin_tree::NodeId;

use crate::{Render, render::RenderContext};

pub struct Text {
    content: String,
    layout: Layout,
    style: Style,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            layout: Layout::default(),
            style: Style::default(),
        }
    }

    #[must_use]
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }
}

pub struct TextState {
    node_id: NodeId,
}

impl<Ctx: RenderContext + Clone> Render<Ctx> for Text {
    type State = TextState;

    fn build(self, ctx: &mut Ctx) -> Self::State {
        let node_id = ctx
            .create_text(self.content, self.layout, self.style)
            .expect("failed to create text node");

        TextState { node_id }
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
        ctx.update_text(state.node_id, self.content, self.layout, self.style);
    }
}
