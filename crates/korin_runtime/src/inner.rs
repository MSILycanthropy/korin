use korin_event::{Event, EventContext, Listeners};
use korin_focus::FocusManager;
use korin_layout::{Layout, LayoutEngine, Rect, Size};
use korin_style::Style;
use korin_tree::{NodeId, Tree};
use slotmap::SecondaryMap;

use crate::{NodeContent, RuntimeError, error::RuntimeResult, node::Node};

type SlotSet<T> = SecondaryMap<T, ()>;

pub struct RuntimeInner {
    pub tree: Tree<Node>,
    pub layout: LayoutEngine,
    pub focus: FocusManager<NodeId>,
    pub event_listeners: SecondaryMap<NodeId, Listeners>,
    pub focusable: SlotSet<NodeId>,
}

impl RuntimeInner {
    pub fn new() -> Self {
        Self {
            tree: Tree::new(),
            layout: LayoutEngine::new(),
            focus: FocusManager::new(),
            event_listeners: SecondaryMap::new(),
            focusable: SlotSet::new(),
        }
    }

    pub fn create_node(&mut self, node: Node, layout: Layout) -> RuntimeResult<NodeId> {
        let text = if let NodeContent::Text(text) = &node.content {
            Some(text.clone())
        } else {
            None
        };
        let node_id = self.tree.new_leaf(node);

        if let Some(ref text) = text {
            self.layout.insert_text(layout, node_id, text)?;
            tracing::debug!(node = %node_id, text = text, "create_node");
        } else {
            self.layout.insert(layout, node_id)?;
            tracing::debug!(node = %node_id, "create_node");
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
        self.event_listeners.remove(id);
        self.focusable.remove(id);

        tracing::debug!(node = %id, "remove_node");

        Ok(())
    }

    pub fn set_focusable(&mut self, id: NodeId) {
        self.focusable.insert(id, ());

        tracing::trace!(node = %id, "set_focus");
    }

    pub fn event_listeners_mut(&mut self, id: NodeId) -> &mut Listeners {
        if !self.event_listeners.contains_key(id) {
            self.event_listeners.insert(id, Listeners::new());
        }

        &mut self.event_listeners[id]
    }

    pub fn add_listener<E: Event>(
        &mut self,
        id: NodeId,
        handler: impl Fn(&EventContext<E>) + Send + Sync + 'static,
    ) {
        self.event_listeners_mut(id).add(handler);
        tracing::trace!(node = %id, event = std::any::type_name::<E>(), "add_listener");
    }

    pub fn emit<E: Event>(&self, id: NodeId, event: &E) -> bool {
        let Some(listeners) = self.event_listeners.get(id) else {
            return false;
        };

        let stopped = listeners.emit(event);
        tracing::trace!(node = %id, event = std::any::type_name::<E>(), stopped, "emit");

        stopped
    }

    pub fn cascade_styles(&mut self, node_id: NodeId, inherited: &Style) -> RuntimeResult<()> {
        let Some(node) = self.get_mut(node_id) else {
            tracing::warn!(node = %node_id, "cascade_styles failed: node not found");
            return Err(RuntimeError::NodeNotFound(node_id));
        };

        node.computed_style = node.style.clone().merge(inherited);
        let computed = node.computed_style.clone();

        for child_id in self.tree.children(node_id) {
            self.cascade_styles(child_id, &computed)?;
        }

        Ok(())
    }

    pub fn compute_layout(&mut self, size: Size) -> RuntimeResult<()> {
        let _span =
            tracing::debug_span!("compute_layout", width = size.width, height = size.height)
                .entered();

        if let Some(root) = self.root() {
            self.cascade_styles(root, &Style::default())?;
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
                if self.focusable.contains_key(id) {
                    order.push(id);
                }
            });
        }

        tracing::debug!(focusable_count = order.len(), "update_focus_order");
        self.focus.set_order(order);
    }
}

impl Default for RuntimeInner {
    fn default() -> Self {
        Self::new()
    }
}
