use std::collections::{HashMap, HashSet};

use korin_focus::FocusManager;
use korin_layout::{Layout, LayoutEngine, Rect, Size};
use korin_style::Style;
use korin_tree::{NodeId, Tree};
use korin_view::{EventHandler, FocusHandler};

use crate::{NodeContent, RuntimeError, error::RuntimeResult, node::Node};

pub struct FocusCallbacks {
    pub on_focus: Option<FocusHandler>,
    pub on_blur: Option<FocusHandler>,
}

pub struct RuntimeInner {
    pub tree: Tree<Node>,
    pub layout: LayoutEngine,
    pub focus: FocusManager<NodeId>,

    pub event_handlers: HashMap<NodeId, EventHandler>,
    pub focus_callbacks: HashMap<NodeId, FocusCallbacks>,
    pub focusable: HashSet<NodeId>,
}

impl RuntimeInner {
    pub fn new() -> Self {
        Self {
            tree: Tree::new(),
            layout: LayoutEngine::new(),
            focus: FocusManager::new(),
            event_handlers: HashMap::new(),
            focus_callbacks: HashMap::new(),
            focusable: HashSet::new(),
        }
    }

    pub fn create_node(&mut self, node: Node, layout: Layout) -> RuntimeResult<NodeId> {
        let text = if let NodeContent::Text(text) = &node.content {
            Some(text.clone())
        } else {
            None
        };
        let node_id = self.tree.new_leaf(node);

        if let Some(text) = text {
            self.layout.insert_text(layout, node_id, text)?;
        } else {
            self.layout.insert(layout, node_id)?;
        }

        Ok(node_id)
    }

    pub fn set_root(&mut self, id: NodeId) -> RuntimeResult<()> {
        self.tree.set_root(id)?;

        Ok(())
    }

    pub fn append_child(&mut self, parent: NodeId, child: NodeId) -> RuntimeResult<()> {
        self.tree.append(parent, child)?;
        self.layout.append(parent, child)?;

        Ok(())
    }

    pub fn remove_node(&mut self, id: NodeId) -> RuntimeResult<()> {
        self.tree.remove(id)?;
        self.layout.remove(id)?;
        self.event_handlers.remove(&id);
        self.focus_callbacks.remove(&id);
        self.focusable.remove(&id);

        Ok(())
    }

    pub fn set_focusable(&mut self, id: NodeId) {
        self.focusable.insert(id);
    }

    pub fn set_event_handler(&mut self, id: NodeId, handler: EventHandler) {
        self.event_handlers.insert(id, handler);
    }

    pub fn set_focus_callbacks(
        &mut self,
        id: NodeId,
        on_focus: Option<FocusHandler>,
        on_blur: Option<FocusHandler>,
    ) {
        self.focus_callbacks
            .insert(id, FocusCallbacks { on_focus, on_blur });
    }

    pub fn try_on_blur(&self, id: NodeId) -> RuntimeResult<()> {
        let Some(callbacks) = self.focus_callbacks.get(&id) else {
            return Err(RuntimeError::NodeNotFound(id));
        };

        let Some(ref on_blur) = callbacks.on_blur else {
            return Err(RuntimeError::NoHandler);
        };

        on_blur();

        Ok(())
    }

    pub fn try_on_focus(&self, id: NodeId) -> RuntimeResult<()> {
        let Some(callbacks) = self.focus_callbacks.get(&id) else {
            return Err(RuntimeError::NodeNotFound(id));
        };

        let Some(ref on_focus) = callbacks.on_focus else {
            return Err(RuntimeError::NoHandler);
        };

        on_focus();

        Ok(())
    }

    pub fn try_on_event<E: 'static>(&self, id: NodeId, event: &E) -> RuntimeResult<()> {
        let Some(on_event) = self.event_handlers.get(&id) else {
            return Err(RuntimeError::NoHandler);
        };

        on_event.call::<E>(event);

        Ok(())
    }

    pub fn cascade_styles(&mut self, node_id: NodeId, inherited: Style) -> RuntimeResult<()> {
        let Some(node) = self.get_mut(node_id) else {
            return Err(RuntimeError::NodeNotFound(node_id));
        };

        node.computed_style = node.style.merge(&inherited);
        let computed = node.computed_style;

        for child_id in self.tree.children(node_id) {
            self.cascade_styles(child_id, computed)?;
        }

        Ok(())
    }

    pub fn compute_layout(&mut self, size: Size) -> RuntimeResult<()> {
        if let Some(root) = self.root() {
            self.cascade_styles(root, Style::new())?;
        }

        self.layout.compute(&self.tree, size)?;

        Ok(())
    }

    pub fn rect(&self, id: NodeId) -> Option<Rect> {
        self.layout.rect(id)
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.tree.get(id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.tree.get_mut(id)
    }

    pub const fn root(&self) -> Option<NodeId> {
        self.tree.root()
    }

    pub fn focused(&self) -> Option<NodeId> {
        self.focus.focused().or_else(|| self.root())
    }

    pub fn children(&self, id: NodeId) -> Vec<NodeId> {
        self.tree.children(id)
    }

    pub fn update_focus_order(&mut self) {
        let mut order = Vec::new();

        if let Some(root) = self.root() {
            self.tree.traverse(root, |id, _| {
                if self.focusable.contains(&id) {
                    order.push(id);
                }
            });
        }

        self.focus.set_order(order);
    }
}

impl Default for RuntimeInner {
    fn default() -> Self {
        Self::new()
    }
}
