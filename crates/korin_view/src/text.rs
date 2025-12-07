use korin_tree::NodeId;

use crate::{Render, render::RenderContext};

pub struct Text {
    content: String,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }

    #[must_use] 
    pub fn content(&self) -> &str {
        &self.content
    }
}

pub struct TextState {
    node_id: NodeId,
}

impl Render for Text {
    type State = TextState;

    fn build(self, _ctx: &mut RenderContext) -> Self::State {
        // TODO: Create text node in layout engine
        TextState {
            node_id: NodeId::default(), // placeholder
        }
    }

    fn rebuild(self, _state: &mut Self::State, _ctx: &mut RenderContext) {
        // TODO: Update text content
    }
}
