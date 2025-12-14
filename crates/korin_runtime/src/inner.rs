use korin_event::{Blur, Event, EventContext, Focus, Listeners};
use korin_focus::FocusManager;
use korin_layout::{Layout, LayoutEngine, LayoutInfo, Overflow, Point, Rect, Size};
use korin_style::{Borders, Style, WhiteSpace};
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
        let wrap = node.computed_style.white_space() == WhiteSpace::Normal;
        let node_id = self.tree.new_leaf(node);

        if let Some(ref text) = text {
            self.layout.insert_text(layout, node_id, text, wrap)?;
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

        self.layout.compute(&self.tree, size, |node: &Node| {
            let borders = node.computed_style.borders();

            LayoutInfo {
                border_left: f32::from(borders.contains(Borders::LEFT)),
                border_top: f32::from(borders.contains(Borders::TOP)),
                border_right: f32::from(borders.contains(Borders::RIGHT)),
                border_bottom: f32::from(borders.contains(Borders::BOTTOM)),
                clip_x: node.computed_style.overflow_x() != Overflow::Visible,
                clip_y: node.computed_style.overflow_y() != Overflow::Visible,
                scroll_x: node.scroll_offset.x,
                scroll_y: node.scroll_offset.y,
                scrollbar_width: f32::from(node.computed_style.overflow_y() == Overflow::Scroll),
                scrollbar_height: 0.0, // TODO: horizontal scrolling
            }
        })?;

        if let Some(root) = self.root() {
            self.populate_content_sizes(root);
        }

        Ok(())
    }

    fn populate_content_sizes(&mut self, node_id: NodeId) {
        self.tree.traverse_mut(node_id, |id, node| {
            if let Some(size) = self.layout.content_size(id) {
                node.content_size = size;
            }
        });
    }

    #[must_use]
    pub fn hit_test(&self, point: Point) -> Option<NodeId> {
        self.layout
            .hit_test(&self.tree, point, |node| node.computed_style.z_index())
    }

    #[must_use]
    pub fn find_scrollable_ancestor(&self, id: NodeId) -> Option<NodeId> {
        let mut current = Some(id);

        while let Some(node_id) = current {
            if self.is_scrollable(node_id) {
                return Some(node_id);
            }

            current = self.tree.parent(node_id);
        }

        None
    }

    #[must_use]
    pub fn find_focusable_ancestor(&self, id: NodeId) -> Option<NodeId> {
        let mut current = Some(id);

        while let Some(node_id) = current {
            if self.focusable.contains_key(node_id) {
                return Some(node_id);
            }

            current = self.tree.parent(node_id);
        }

        None
    }

    #[must_use]
    pub fn is_scrollable(&self, id: NodeId) -> bool {
        let Some(node) = self.get(id) else {
            return false;
        };

        let overflow_x = node.computed_style.overflow_x();
        let overflow_y = node.computed_style.overflow_y();

        if overflow_x == Overflow::Scroll || overflow_y == Overflow::Scroll {
            return true;
        }

        false
    }

    pub(crate) fn scroll(&mut self, position: Point, delta: Point) {
        let Some(hit) = self
            .layout
            .hit_test(&self.tree, position, |n| n.computed_style.z_index())
        else {
            return;
        };

        let Some(scrollable) = self.find_scrollable_ancestor(hit) else {
            return;
        };

        let max = self.layout.max_scroll(scrollable).unwrap_or_default();

        let Some(node) = self.get_mut(scrollable) else {
            return;
        };

        node.scroll_offset.x = (node.scroll_offset.x + delta.x).clamp(0.0, max.x);
        node.scroll_offset.y = (node.scroll_offset.y + delta.y).clamp(0.0, max.y);

        tracing::debug!(
            node = hit.to_string(),
            delta = ?delta,
            scroll_offset = ?node.scroll_offset,
            "scroll"
        );
    }

    pub(crate) fn move_focus(&mut self, reverse: bool) {
        let change = if reverse {
            self.focus.focus_prev()
        } else {
            self.focus.focus_next()
        };

        if !change.relevant() {
            return;
        }

        tracing::debug!(
            prev = ?change.prev().map(|id| id.to_string()),
            next = ?change.next().map(|id| id.to_string()),
            "focus_changed"
        );

        if let Some(prev) = change.prev() {
            self.emit(prev, &Blur);
        }

        if let Some(next) = change.next() {
            self.emit(next, &Focus);
        }
    }

    pub(crate) fn mouse_down(&mut self, position: Point) {
        let Some(hit) = self
            .layout
            .hit_test(&self.tree, position, |n| n.computed_style.z_index())
        else {
            return;
        };

        let Some(focusable) = self.find_focusable_ancestor(hit) else {
            return;
        };

        let current = self.focus.focused();
        if current == Some(focusable) {
            return;
        }

        if let Some(prev) = current {
            self.emit(prev, &Blur);
        }

        tracing::debug!(
            prev = ?current.map(|id| id.to_string()),
            next = ?focusable.to_string(),
            "focus_changed"
        );

        self.focus.focus(focusable);
        self.emit(focusable, &Focus);
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
