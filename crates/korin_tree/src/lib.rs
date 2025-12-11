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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tree_is_empty() {
        let tree: Tree<i32> = Tree::new();
        assert!(tree.root().is_none());
    }

    #[test]
    fn new_leaf_returns_valid_id() {
        let mut tree = Tree::new();
        let id = tree.new_leaf(42);
        assert!(tree.contains(id));
        assert_eq!(tree.get(id), Some(&42));
    }

    #[test]
    fn set_root_works() {
        let mut tree = Tree::new();
        let id = tree.new_leaf(1);
        assert!(tree.set_root(id).is_ok());
        assert_eq!(tree.root(), Some(id));
    }

    #[test]
    fn set_root_fails_for_invalid_id() {
        let mut tree: Tree<i32> = Tree::new();
        let other_tree_id = {
            let mut other = Tree::new();
            other.new_leaf(1)
        };
        assert!(tree.set_root(other_tree_id).is_err());
    }

    #[test]
    fn append_adds_child() {
        let mut tree = Tree::new();
        let parent = tree.new_leaf(1);
        let child = tree.new_leaf(2);

        assert!(tree.append(parent, child).is_ok());
        assert_eq!(tree.children(parent), vec![child]);
    }

    #[test]
    fn append_fails_for_invalid_parent() {
        let mut tree: Tree<i32> = Tree::new();
        let parent = tree.new_leaf(0);
        let child = tree.new_leaf(1);

        tree.remove(parent).expect("remove parent");

        assert!(tree.append(parent, child).is_err());
    }

    #[test]
    fn remove_returns_data() {
        let mut tree = Tree::new();
        let id = tree.new_leaf(42);

        let removed = tree.remove(id);
        assert_eq!(removed.expect("failed"), 42);
        assert!(!tree.contains(id));
    }

    #[test]
    fn remove_fails_for_invalid_id() {
        let mut tree: Tree<i32> = Tree::new();
        let fake_id = {
            let mut other = Tree::new();
            other.new_leaf(1)
        };

        assert!(tree.remove(fake_id).is_err());
    }

    #[test]
    fn remove_also_removes_descendants() {
        let mut tree = Tree::new();
        let root = tree.new_leaf(1);
        let child = tree.new_leaf(2);
        let grandchild = tree.new_leaf(3);

        tree.append(root, child).expect("failed");
        tree.append(child, grandchild).expect("failed");

        tree.remove(child).expect("failed");

        assert!(!tree.contains(child));
        assert!(!tree.contains(grandchild));
        assert!(tree.contains(root));
    }

    #[test]
    fn children_returns_empty_for_leaf() {
        let mut tree = Tree::new();
        let id = tree.new_leaf(1);
        assert!(tree.children(id).is_empty());
    }

    #[test]
    fn children_returns_direct_children_only() {
        let mut tree = Tree::new();
        let root = tree.new_leaf(1);
        let child1 = tree.new_leaf(2);
        let child2 = tree.new_leaf(3);
        let grandchild = tree.new_leaf(4);

        tree.append(root, child1).expect("failed");
        tree.append(root, child2).expect("failed");
        tree.append(child1, grandchild).expect("failed");

        let children = tree.children(root);
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1));
        assert!(children.contains(&child2));
        assert!(!children.contains(&grandchild));
    }

    #[test]
    fn descendants_returns_all_nested() {
        let mut tree = Tree::new();
        let root = tree.new_leaf(1);
        let child = tree.new_leaf(2);
        let grandchild = tree.new_leaf(3);

        tree.append(root, child).expect("failed");
        tree.append(child, grandchild).expect("failed");

        let desc = tree.descendants(root);
        assert_eq!(desc.len(), 2);
        assert!(desc.contains(&child));
        assert!(desc.contains(&grandchild));
    }

    #[test]
    fn descendants_empty_for_leaf() {
        let mut tree = Tree::new();
        let id = tree.new_leaf(1);
        assert!(tree.descendants(id).is_empty());
    }

    #[test]
    fn ancestors_returns_path_to_root() {
        let mut tree = Tree::new();
        let root = tree.new_leaf(1);
        let child = tree.new_leaf(2);
        let grandchild = tree.new_leaf(3);

        tree.set_root(root).expect("failed");

        let child_node = tree.nodes.get_mut(child).expect("failed");
        child_node.parent = Some(root);

        let grandchild_node = tree.nodes.get_mut(grandchild).expect("failed");
        grandchild_node.parent = Some(child);

        tree.append(root, child).expect("failed");
        tree.append(child, grandchild).expect("failed");

        let anc = tree.ancestors(grandchild);
        assert_eq!(anc, vec![child, root]);
    }

    #[test]
    fn ancestors_empty_for_root() {
        let mut tree = Tree::new();
        let root = tree.new_leaf(1);
        tree.set_root(root).expect("failed");
        assert!(tree.ancestors(root).is_empty());
    }

    #[test]
    fn traverse_visits_all_nodes() {
        let mut tree = Tree::new();
        let root = tree.new_leaf(1);
        let child1 = tree.new_leaf(2);
        let child2 = tree.new_leaf(3);

        tree.append(root, child1).expect("failed");
        tree.append(root, child2).expect("failed");

        let mut visited = Vec::new();
        tree.traverse(root, |_, data| visited.push(*data));

        assert_eq!(visited.len(), 3);
        assert!(visited.contains(&1));
        assert!(visited.contains(&2));
        assert!(visited.contains(&3));
    }

    #[test]
    fn traverse_mut_can_modify() {
        let mut tree = Tree::new();
        let root = tree.new_leaf(1);
        let child = tree.new_leaf(2);

        tree.append(root, child).expect("failed");
        tree.traverse_mut(root, |_, data| *data *= 10);

        assert_eq!(tree.get(root), Some(&10));
        assert_eq!(tree.get(child), Some(&20));
    }

    #[test]
    fn get_mut_allows_modification() {
        let mut tree = Tree::new();
        let id = tree.new_leaf(1);

        if let Some(data) = tree.get_mut(id) {
            *data = 99;
        }

        assert_eq!(tree.get(id), Some(&99));
    }
}
