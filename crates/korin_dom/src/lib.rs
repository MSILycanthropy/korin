mod document;
mod element;
mod node;

use slotmap::new_key_type;

new_key_type! {
    pub struct NodeId;
}
