use indextree::NodeId;

use crate::document::Document;

/// Trait for view states that can be mounted/unmounted from the DOM.
///
/// This separates node creation from DOM attachment, enabling efficient
/// list reconciliation where items can be moved/removed independently.
pub trait Mountable {
    /// Attach nodes to parent, inserting before `marker` (or appending if None).
    fn mount(&mut self, parent: NodeId, marker: Option<NodeId>, document: &mut Document);

    /// Detach nodes from DOM without destroying them.
    fn unmount(&mut self, document: &mut Document);

    /// Returns the first DOM node, used for positioning.
    fn first_node(&self) -> Option<NodeId>;
}

/// Unit type is trivially mountable (no nodes)
impl Mountable for () {
    fn mount(&mut self, _parent: NodeId, _marker: Option<NodeId>, _document: &mut Document) {}

    fn unmount(&mut self, _doc: &mut Document) {}

    fn first_node(&self) -> Option<NodeId> {
        None
    }
}
