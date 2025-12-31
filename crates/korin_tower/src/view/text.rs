use indextree::NodeId;

use crate::{
    document::Document,
    view::{
        Mountable, View,
        context::{BuildContext, RebuildContext},
    },
};

/// A static text view
pub struct TextView {
    content: String,
}

impl TextView {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

impl From<&str> for TextView {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for TextView {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

pub struct TextViewState {
    node: NodeId,
}

impl View for TextView {
    type State = TextViewState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        let node = ctx.create_text(self.content);
        TextViewState { node }
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RebuildContext) {
        ctx.set_text(state.node, self.content);
    }
}

impl Mountable for TextViewState {
    fn mount(&mut self, parent: NodeId, marker: Option<NodeId>, document: &mut Document) {
        match marker {
            Some(marker) => document.insert_before(marker, self.node),
            None => document.append_child(parent, self.node),
        }
    }

    fn unmount(&mut self, document: &mut Document) {
        document.detach(self.node);
    }

    fn first_node(&self) -> Option<NodeId> {
        Some(self.node)
    }
}
