use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use korin_layout::Layout;
use korin_style::Style;
use korin_tree::NodeId;
use korin_view::{EventHandler, FocusHandler, RenderContext};

use crate::{Node, NodeContent, RuntimeResult, inner::RuntimeInner};

#[derive(Clone)]
pub struct RuntimeContext {
    runtime: Arc<RwLock<RuntimeInner>>,
    parent: Option<NodeId>,
}

impl RuntimeContext {
    pub const fn new(runtime: Arc<RwLock<RuntimeInner>>) -> Self {
        Self {
            runtime,
            parent: None,
        }
    }

    #[must_use]
    pub fn with_parent(&self, parent: NodeId) -> Self {
        Self {
            runtime: self.runtime.clone(),
            parent: Some(parent),
        }
    }

    /// Returns the runtime of this [`RuntimeContext`].
    ///
    /// # Panics
    ///
    /// Panics if the `runtime` is poisoned.
    pub fn runtime(&self) -> RwLockReadGuard<'_, RuntimeInner> {
        self.runtime.read().expect("poisoned")
    }

    /// Returns the runtime of this [`RuntimeContext`].
    ///
    /// # Panics
    ///
    /// Panics if the `runtime` is poisoned.
    pub fn runtime_mut(&mut self) -> RwLockWriteGuard<'_, RuntimeInner> {
        self.runtime.write().expect("poisoned")
    }

    #[must_use]
    pub const fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    pub fn set_root(&mut self, id: NodeId) -> RuntimeResult<()> {
        self.runtime_mut().set_root(id)
    }
}

impl RenderContext for RuntimeContext {
    fn create_container(&mut self, layout: Layout, style: Style) -> Option<NodeId> {
        let node = Node::container(style);
        let node = self.runtime_mut().create_node(node, layout).ok()?;

        if let Some(parent) = self.parent {
            self.runtime_mut().append_child(parent, node).ok()?;
        } else {
            self.runtime_mut().set_root(node).ok()?;
        }

        Some(node)
    }

    fn update_container(&mut self, id: NodeId, layout: Layout, style: Style) {
        let mut runtime = self.runtime_mut();

        if let Some(node) = runtime.tree.get_mut(id) {
            node.style = style;
        }

        runtime
            .layout
            .update(id, layout)
            .expect("updating container failed");
    }

    fn create_text(&mut self, content: String, layout: Layout, style: Style) -> Option<NodeId> {
        let node = Node::text(content, style);
        let node = self.runtime_mut().create_node(node, layout).ok()?;

        if let Some(parent) = self.parent {
            self.runtime_mut().append_child(parent, node).ok()?;
        } else {
            self.runtime_mut().set_root(node).ok()?;
        }

        Some(node)
    }

    fn update_text(&mut self, id: NodeId, content: String, layout: Layout, style: Style) {
        let mut runtime = self.runtime_mut();

        if let Some(node) = runtime.tree.get_mut(id) {
            node.content = NodeContent::Text(content.clone());
            node.style = style;
        }

        runtime
            .layout
            .update_text(id, layout, content)
            .expect("updating text failed");
    }

    fn update_style(&mut self, id: NodeId, style: Style) {
        let mut runtime = self.runtime_mut();

        if let Some(node) = runtime.tree.get_mut(id) {
            node.style = style;
        }
    }

    fn set_focusable(&mut self, id: NodeId) {
        self.runtime_mut().set_focusable(id);
    }

    fn set_event_handler(&mut self, id: NodeId, handler: EventHandler) {
        self.runtime_mut().set_event_handler(id, handler);
    }

    fn set_focus_callbacks(
        &mut self,
        id: NodeId,
        on_focus: Option<FocusHandler>,
        on_blur: Option<FocusHandler>,
    ) {
        self.runtime_mut()
            .set_focus_callbacks(id, on_focus, on_blur);
    }

    fn with_parent(&self, parent: NodeId) -> Self {
        Self {
            runtime: self.runtime.clone(),
            parent: Some(parent),
        }
    }
}
