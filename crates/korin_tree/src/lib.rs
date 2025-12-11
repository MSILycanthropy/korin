use slotmap::{Key, SlotMap, new_key_type};
use thiserror::Error;

new_key_type! {
  pub struct NodeId;
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}", self.data().as_ffi()))
    }
}

#[derive(Error, Debug)]
pub enum TreeError {
    #[error("node not found: {0}")]
    NodeNotFound(NodeId),
}

pub type TreeResult<T> = Result<T, TreeError>;

pub struct Node<T> {
    pub data: T,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

impl<T> Node<T> {
    const fn from(data: T) -> Self {
        Self {
            data,
            parent: None,
            children: Vec::new(),
        }
    }
}

pub struct Tree<T> {
    nodes: SlotMap<NodeId, Node<T>>,
    root: Option<NodeId>,
}

impl<T> Tree<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            root: None,
        }
    }

    #[must_use]
    pub const fn root(&self) -> Option<NodeId> {
        self.root
    }

    pub fn set_root(&mut self, id: NodeId) -> TreeResult<()> {
        if !self.nodes.contains_key(id) {
            tracing::warn!(node = %id, "set_root failed: node not found");
            return Err(TreeError::NodeNotFound(id));
        }

        tracing::debug!(node = %id, "set_root");
        self.root = Some(id);

        Ok(())
    }

    pub fn new_leaf(&mut self, data: T) -> NodeId {
        let id = self.nodes.insert(Node::from(data));
        tracing::debug!(node = %id, "new_leaf");
        id
    }

    pub fn append(&mut self, parent: NodeId, child: NodeId) -> TreeResult<()> {
        let Some(parent_node) = self.nodes.get_mut(parent) else {
            tracing::warn!(parent = %parent, child = %child, "append failed: parent not found");
            return Err(TreeError::NodeNotFound(parent));
        };

        parent_node.children.push(child);

        tracing::debug!(parent = %parent, child = %child, "append");

        Ok(())
    }

    pub fn remove(&mut self, id: NodeId) -> TreeResult<T> {
        let descendants = self.descendants(id);

        if let Some(parent) = self
            .nodes
            .get(id)
            .and_then(|n| n.parent)
            .and_then(|n| self.nodes.get_mut(n))
        {
            parent.children.retain(|&c| c != id);
        }

        let Some(removed) = self.nodes.remove(id).map(|n| n.data) else {
            tracing::warn!(node = %id, "remove failed: node not found");
            return Err(TreeError::NodeNotFound(id));
        };

        let count = descendants.len();
        for d in descendants {
            self.nodes.remove(d);
        }

        tracing::debug!(node = %id, descendants = count, "remove");

        Ok(removed)
    }

    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&T> {
        self.nodes.get(id).map(|n| &n.data)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut T> {
        self.nodes.get_mut(id).map(|n| &mut n.data)
    }

    #[must_use]
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    #[must_use]
    pub fn children(&self, id: NodeId) -> Vec<NodeId> {
        self.nodes
            .get(id)
            .map(|n| n.children.clone())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn descendants(&self, id: NodeId) -> Vec<NodeId> {
        let mut result = vec![];
        let mut stack = vec![id];

        while let Some(node) = stack.pop().and_then(|current| self.nodes.get(current)) {
            for &child in &node.children {
                result.push(child);
                stack.push(child);
            }
        }

        result
    }

    #[must_use]
    pub fn ancestors(&self, id: NodeId) -> Vec<NodeId> {
        let mut result = Vec::new();
        let mut current = id;

        while let Some(node) = self.nodes.get(current) {
            if let Some(parent) = node.parent {
                result.push(parent);
                current = parent;
            } else {
                break;
            }
        }

        result
    }

    pub fn traverse<F>(&self, id: NodeId, mut f: F)
    where
        F: FnMut(NodeId, &T),
    {
        let mut stack = vec![id];

        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.get(current) {
                f(current, &node.data);

                for &child in node.children.iter().rev() {
                    stack.push(child);
                }
            }
        }
    }

    pub fn traverse_mut<F>(&mut self, id: NodeId, mut f: F)
    where
        F: FnMut(NodeId, &mut T),
    {
        let mut stack = vec![id];

        while let Some(current) = stack.pop() {
            if let Some(node) = self.nodes.get_mut(current) {
                f(current, &mut node.data);

                for &child in node.children.iter().rev() {
                    stack.push(child);
                }
            }
        }
    }
}

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}
