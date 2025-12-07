use std::collections::HashMap;

use korin_tree::{NodeId, Tree};
use taffy::{NodeId as TaffyId, TaffyTree};

use crate::{Layout, LayoutResult, Rect, Size, error::LayoutError};

pub struct Engine {
    taffy: TaffyTree,
    nodes: HashMap<NodeId, TaffyId>,
}

impl Engine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            nodes: HashMap::new(),
        }
    }

    pub fn create_node(&mut self, layout: Layout) -> LayoutResult<TaffyId> {
        let id = self.taffy.new_leaf(layout.into())?;

        Ok(id)
    }

    pub fn insert(&mut self, id: NodeId, taffy_id: TaffyId) {
        self.nodes.insert(id, taffy_id);
    }

    pub fn remove(&mut self, id: NodeId) {
        if let Some(taffy_id) = self.nodes.remove(&id) {
            let _ = self.taffy.remove(taffy_id);
        }
    }

    pub fn append(&mut self, parent: NodeId, child: NodeId) -> LayoutResult<()> {
        let parent_taffy = self.taffy_node(parent)?;
        let child_taffy = self.taffy_node(child)?;

        self.taffy.add_child(parent_taffy, child_taffy)?;

        Ok(())
    }

    pub fn compute<T>(&mut self, tree: &Tree<T>, size: Size) -> LayoutResult<()> {
        let root = tree.root().ok_or(LayoutError::NoRoot)?;
        let root_taffy = self.taffy_node(root)?;

        self.taffy.compute_layout(root_taffy, size.into())?;

        Ok(())
    }

    #[must_use]
    pub fn rect(&self, id: NodeId) -> Option<Rect> {
        let taffy_id = self.nodes.get(&id)?;
        let layout = self.taffy.layout(*taffy_id).ok()?;

        Some(layout.into())
    }

    fn taffy_node(&self, id: NodeId) -> LayoutResult<TaffyId> {
        self.nodes
            .get(&id)
            .ok_or(LayoutError::NodeNotFound(id))
            .copied()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
