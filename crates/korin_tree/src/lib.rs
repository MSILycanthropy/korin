use slotmap::{SlotMap, new_key_type};

new_key_type! {
  pub struct NodeId;
}

pub struct Node<T> {
    pub data: T,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

impl<T> Node<T> {
    fn from(data: T) -> Self {
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
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            root: None,
        }
    }

    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    pub fn set_root(&mut self, data: T) -> NodeId {
        let id = self.nodes.insert(Node::from(data));

        self.root = Some(id);

        id
    }

    pub fn append(&mut self, parent: NodeId, data: T) -> Option<NodeId> {
        if !self.contains(parent) {
            return None;
        }

        let id = self.nodes.insert(Node::from(data));

        self.nodes[parent].children.push(id);

        Some(id)
    }

    // TODO: This
    pub fn remove(&mut self, id: NodeId) -> Option<T> {
        let descendants = self.descendants(id);

        if let Some(parent) = self
            .get(id)
            .and_then(|n| n.parent)
            .and_then(|n| self.get_mut(n))
        {
            parent.children.retain(|&c| c != id);
        }

        let removed = self.nodes.remove(id).map(|n| n.data);

        descendants.into_iter().for_each(|d| {
            self.nodes.remove(d);
        });

        removed
    }

    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.nodes.get_mut(id)
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    pub fn descendants(&self, id: NodeId) -> Vec<NodeId> {
        let mut result = vec![];
        let mut stack = vec![id];

        while let Some(node) = stack.pop().and_then(|current| self.get(current)) {
            for &child in &node.children {
                result.push(child);
                stack.push(child);
            }
        }

        result
    }

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
            if let Some(node) = self.get(current) {
                f(current, &node.data);

                for &child in node.children.iter().rev() {
                    stack.push(child)
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
            if let Some(node) = self.get_mut(current) {
                f(current, &mut node.data);

                for &child in node.children.iter().rev() {
                    stack.push(child)
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
