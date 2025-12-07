use std::collections::{HashMap, HashSet};

use korin_focus::FocusManager;
use korin_layout::{Layout, LayoutEngine, Rect, Size};
use korin_tree::{NodeId, Tree};
use korin_view::{EventHandler, FocusHandler};

use crate::{error::RuntimeResult, node::Node};

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
        let node_id = self.tree.new_leaf(node);

        self.layout.insert(layout, node_id)?;

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

    pub fn compute_layout(&mut self, size: Size) -> RuntimeResult<()> {
        self.layout.compute(&self.tree, size)?;

        Ok(())
    }

    pub fn rect(&self, id: NodeId) -> Option<Rect> {
        self.layout.rect(id)
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.tree.get(id)
    }

    pub const fn root(&self) -> Option<NodeId> {
        self.tree.root()
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
