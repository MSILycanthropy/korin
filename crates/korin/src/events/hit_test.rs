use capsule_corp::Layout;
use indextree::NodeId;
use tracing::trace;

use crate::Document;

impl Document {
    // TODO: take z-index into account when hit testing <3
    pub fn hit_test(&self, x: u16, y: u16) -> Option<NodeId> {
        let result = self.hit_test_node(self.root(), x, y);

        trace!(doc = %self.id(), x, y, result = ?result, "hit test");

        result
    }

    fn hit_test_node(&self, id: NodeId, x: u16, y: u16) -> Option<NodeId> {
        let node = self.get(id)?;

        if !node.is_element() {
            return None;
        }

        let layout = node.layout;

        if !is_in_layout(&layout, x, y) {
            return None;
        }

        let children: Vec<NodeId> = self.children(id).collect();

        for &child in children.iter().rev() {
            if let Some(hit) = self.hit_test_node(child, x, y) {
                return Some(hit);
            }
        }

        Some(id)
    }
}

#[inline]
const fn is_in_layout(layout: &Layout, x: u16, y: u16) -> bool {
    let border_box = layout.resolved_box.border_box_size();

    let left = layout.location.x;
    let top = layout.location.y;
    let right = left.saturating_add(border_box.width);
    let bottom = top.saturating_add(border_box.height);

    x >= left && x < right && y >= top && y < bottom
}
