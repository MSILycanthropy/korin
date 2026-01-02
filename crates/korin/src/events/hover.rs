use capsule_corp::ElementState;
use dom_events::EventType;
use indextree::NodeId;
use tracing::debug;

use crate::{Document, Node, events::MouseEvent};

impl Document {
    pub fn update_hover(&mut self, target: Option<NodeId>, mouse_event: &MouseEvent) {
        let old = self.hovered();

        if old == target {
            return;
        }

        debug!(doc = %self.id(), old = ?old, new = ?target, "hover change");

        let old_path: Vec<NodeId> = old
            .map(|node| node.ancestors(&self.arena).collect())
            .unwrap_or_default();
        let new_path: Vec<NodeId> = target
            .map(|node| node.ancestors(&self.arena).collect())
            .unwrap_or_default();

        let common_ancestor = find_common_ancestor(&old_path, &new_path);

        let leave_count = common_ancestor.map_or(old_path.len(), |i| i);

        for &node in old_path.iter().take(leave_count) {
            self.leave_node(node, target, mouse_event);
        }

        let enter_start = common_ancestor.map_or(new_path.len(), |i| i);

        for &node in new_path.iter().take(enter_start).rev() {
            self.enter_node(node, target, mouse_event);
        }

        self.set_hovered(target);
    }

    fn leave_node(&mut self, id: NodeId, related_target: Option<NodeId>, mouse_event: &MouseEvent) {
        if let Some(element) = self.get_mut(id).and_then(Node::as_element_mut) {
            element.remove_state(ElementState::HOVER);
        }

        let event_type = EventType::MouseLeave(MouseEvent {
            related_target,
            ..*mouse_event
        });

        self.dispatch_direct(id, event_type);
    }

    fn enter_node(&mut self, id: NodeId, related_target: Option<NodeId>, mouse_event: &MouseEvent) {
        if let Some(element) = self.get_mut(id).and_then(Node::as_element_mut) {
            element.add_state(ElementState::HOVER);
        }

        let event_type = EventType::MouseEnter(MouseEvent {
            related_target,
            ..*mouse_event
        });

        self.dispatch_direct(id, event_type);
    }
}

fn find_common_ancestor(old_path: &[NodeId], new_path: &[NodeId]) -> Option<usize> {
    if old_path.is_empty() || new_path.is_empty() {
        return None;
    }

    for (index, &old_node) in old_path.iter().enumerate() {
        if new_path.contains(&old_node) {
            return Some(index);
        }
    }

    None
}
