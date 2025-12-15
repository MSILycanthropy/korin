use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use korin_event::Listeners;
use korin_layout::Layout;
use korin_style::{Style, WhiteSpace};
use korin_tree::NodeId;
use korin_view::RenderContext;

use crate::{Node, NodeContent, RuntimeResult, inner::RuntimeInner};

#[derive(Clone)]
pub struct RuntimeContext {
    runtime: Arc<RwLock<RuntimeInner>>,
    parent: Option<NodeId>,
    last_created: Option<NodeId>,
}

impl RuntimeContext {
    pub const fn new(runtime: Arc<RwLock<RuntimeInner>>) -> Self {
        Self {
            runtime,
            parent: None,
            last_created: None,
        }
    }

    #[must_use]
    pub fn with_parent(&self, parent: NodeId) -> Self {
        Self {
            runtime: self.runtime.clone(),
            parent: Some(parent),
            last_created: None,
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
    fn create_container(&mut self) -> Option<NodeId> {
        let node = Node::container();
        let node = self
            .runtime_mut()
            .create_node(node, Layout::default())
            .ok()?;

        if let Some(parent) = self.parent {
            self.runtime_mut().append_child(parent, node).ok()?;
        } else {
            self.runtime_mut().set_root(node).ok()?;
        }

        self.last_created = Some(node);

        Some(node)
    }

    fn update_container(&mut self, _: NodeId) {}

    fn create_text(&mut self, content: String) -> Option<NodeId> {
        let node = Node::text(content);
        let node = self
            .runtime_mut()
            .create_node(node, Layout::default())
            .ok()?;

        eprintln!("create_text: parent={:?}", self.parent);

        if let Some(parent) = self.parent {
            self.runtime_mut().append_child(parent, node).ok()?;
        } else {
            self.runtime_mut().set_root(node).ok()?;
        }

        self.last_created = Some(node);

        Some(node)
    }

    fn update_text(&mut self, id: NodeId, content: String) {
        let mut runtime = self.runtime_mut();

        let mut wrap = true;
        if let Some(node) = runtime.tree.get_mut(id) {
            wrap = node.computed_style.white_space() == WhiteSpace::Normal;
            node.content = NodeContent::Text(content.clone());
        }

        runtime
            .layout
            .update_text(id, content, wrap)
            .expect("updating text failed");
    }

    fn create_style(&mut self, id: NodeId, style: Style) {
        let mut runtime = self.runtime_mut();

        if let Some(node) = runtime.tree.get_mut(id) {
            node.style = style;
        }
    }

    fn update_style(&mut self, id: NodeId, style: Style) {
        let mut runtime = self.runtime_mut();

        if let Some(node) = runtime.tree.get_mut(id) {
            node.style = style.clone();
        }

        let _ = runtime.layout.update(id, style.layout().clone());
    }

    fn set_focusable(&mut self, id: NodeId) {
        self.runtime_mut().set_focusable(id);
    }

    fn set_listeners(&mut self, id: NodeId, listeners: Listeners) {
        self.runtime_mut().event_listeners.insert(id, listeners);
    }

    fn remove_node(&mut self, id: NodeId) {
        let _ = self.runtime_mut().remove_node(id);
    }

    fn last_created_node(&self) -> Option<NodeId> {
        self.last_created
    }

    fn with_parent(&self, parent: NodeId) -> Self {
        Self {
            runtime: self.runtime.clone(),
            parent: Some(parent),
            last_created: None,
        }
    }
}
