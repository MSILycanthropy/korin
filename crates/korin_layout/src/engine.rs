use std::ops::{Deref, DerefMut};

use korin_geometry::Point;
use korin_tree::{NodeId, Tree};
use slotmap::SecondaryMap;
use taffy::{NodeId as TaffyId, TaffyTree};

use crate::{Layout, LayoutResult, Rect, Size, error::LayoutError, measure::taffy_measure};

#[derive(Default, Clone)]
pub struct NodeMeasure(pub Option<String>);

#[expect(unsafe_code, reason = "TaffyTree is safe as long as calc is not used")]
/// SAFETY: Taffy Tree becomes thread unsafe when you use the calc feature, which we do not implement
unsafe impl Send for NodeMeasure {}

#[expect(unsafe_code, reason = "TaffyTree is safe as long as calc is not used")]
/// SAFETY: Taffy Tree becomes thread unsafe when you use the calc feature, which we do not implement
unsafe impl Sync for NodeMeasure {}

pub struct LayoutTree<T: Send + Sync>(TaffyTree<T>);

impl<T: Send + Sync> LayoutTree<T> {
    pub fn new() -> Self {
        Self(TaffyTree::new())
    }
}

#[expect(unsafe_code, reason = "TaffyTree is safe as long as calc is not used")]
#[allow(clippy::non_send_fields_in_send_ty)]
/// SAFETY: Taffy Tree becomes thread unsafe when you use the calc feature, which we do not implement
unsafe impl<T: Send + Sync> Send for LayoutTree<T> {}

#[expect(unsafe_code, reason = "TaffyTree is safe as long as calc is not used")]
/// SAFETY: Taffy Tree becomes thread unsafe when you use the calc feature, which we do not implement
unsafe impl<T: Send + Sync> Sync for LayoutTree<T> {}

impl<T: Send + Sync> Deref for LayoutTree<T> {
    type Target = TaffyTree<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Send + Sync> DerefMut for LayoutTree<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct LayoutInfo {
    pub border_left: f32,
    pub border_top: f32,
    pub border_right: f32,
    pub border_bottom: f32,
    pub clip_x: bool,
    pub clip_y: bool,
}

pub struct Engine {
    taffy: LayoutTree<NodeMeasure>,
    nodes: SecondaryMap<NodeId, TaffyId>,
    computed_rects: SecondaryMap<NodeId, Rect>,
    clip_rects: SecondaryMap<NodeId, Rect>,
}

impl Engine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            taffy: LayoutTree::new(),
            nodes: SecondaryMap::new(),
            computed_rects: SecondaryMap::new(),
            clip_rects: SecondaryMap::new(),
        }
    }

    pub fn insert(&mut self, layout: Layout, node_id: NodeId) -> LayoutResult<TaffyId> {
        let ctx = NodeMeasure(None);
        let taffy_id = self.taffy.new_leaf_with_context(layout.into(), ctx)?;

        self.nodes.insert(node_id, taffy_id);
        tracing::debug!(node = %node_id, "insert");

        Ok(taffy_id)
    }

    pub fn insert_text(
        &mut self,
        layout: Layout,
        node_id: NodeId,
        text: &str,
    ) -> LayoutResult<TaffyId> {
        let text_len = text.len();
        let ctx = NodeMeasure(Some(text.into()));
        let taffy_id = self.taffy.new_leaf_with_context(layout.into(), ctx)?;

        self.nodes.insert(node_id, taffy_id);
        tracing::debug!(node = %node_id, text_len, "insert_text");

        Ok(taffy_id)
    }

    pub fn remove(&mut self, id: NodeId) -> LayoutResult<TaffyId> {
        if let Some(taffy_id) = self.nodes.remove(id) {
            let taffy_id = self.taffy.remove(taffy_id)?;
            tracing::debug!(node = %id, "remove");
            return Ok(taffy_id);
        }

        tracing::warn!(node = %id, "remove failed: node not found");
        Err(LayoutError::NodeNotFound(id))
    }

    pub fn append(&mut self, parent: NodeId, child: NodeId) -> LayoutResult<()> {
        let parent_taffy = self.taffy_node(parent)?;
        let child_taffy = self.taffy_node(child)?;

        self.taffy.add_child(parent_taffy, child_taffy)?;
        tracing::debug!(parent = %parent, child = %child, "append");

        Ok(())
    }

    pub fn update(&mut self, id: NodeId, layout: Layout) -> LayoutResult<()> {
        if let Some(&taffy_id) = self.nodes.get(id) {
            self.taffy.set_style(taffy_id, layout.into())?;
            self.taffy.mark_dirty(taffy_id)?;
            tracing::trace!(node = %id, "update");

            return Ok(());
        }

        tracing::warn!(node = %id, "update failed: node not found");

        Ok(())
    }

    pub fn update_text(&mut self, id: NodeId, text: String) -> LayoutResult<()> {
        if let Some(&taffy_id) = self.nodes.get(id) {
            let log_text = text.clone();
            self.taffy
                .set_node_context(taffy_id, Some(NodeMeasure(Some(text))))?;
            self.taffy.mark_dirty(taffy_id)?;
            tracing::trace!(node = %id, text = log_text, "update_text");

            return Ok(());
        }
        tracing::warn!(node = %id, "update_text failed: node not found");

        Ok(())
    }

    pub fn compute<T, F>(&mut self, tree: &Tree<T>, size: Size, info_fn: F) -> LayoutResult<()>
    where
        F: Fn(&T) -> LayoutInfo,
    {
        let _span =
            tracing::debug_span!("compute", width = size.width, height = size.height).entered();

        let root = tree.root().ok_or(LayoutError::NoRoot)?;
        let root_taffy = self.taffy_node(root)?;

        self.taffy
            .compute_layout_with_measure(root_taffy, size.into(), taffy_measure)?;

        self.computed_rects.clear();
        self.clip_rects.clear();

        let viewport = Rect::new(0.0, 0.0, size.width, size.height);
        self.compute_absolute_rects(tree, root, Point::default(), viewport, &info_fn)?;

        Ok(())
    }

    #[must_use]
    pub fn rect(&self, id: NodeId) -> Option<Rect> {
        let taffy_id = self.nodes.get(id)?;
        let layout = self.taffy.layout(*taffy_id).ok()?;

        Some(layout.into())
    }

    #[must_use]
    pub fn absolute_rect(&self, id: NodeId) -> Option<Rect> {
        self.computed_rects.get(id).copied()
    }

    #[must_use]
    pub fn clip_rect(&self, id: NodeId) -> Option<Rect> {
        self.clip_rects.get(id).copied()
    }

    fn compute_absolute_rects<T, F>(
        &mut self,
        tree: &Tree<T>,
        node_id: NodeId,
        parent_position: Point,
        parent_clip: Rect,
        info_fn: &F,
    ) -> LayoutResult<()>
    where
        F: Fn(&T) -> LayoutInfo,
    {
        let Some(node_data) = tree.get(node_id) else {
            return Ok(());
        };

        let rect = self
            .rect(node_id)
            .ok_or(LayoutError::NodeNotFound(node_id))?;
        let info = info_fn(node_data);

        let abs_rect = Rect::new(
            parent_position.x + rect.x,
            parent_position.y + rect.y,
            rect.width,
            rect.height,
        );

        let clipped = abs_rect.intersect(&parent_clip);

        self.computed_rects.insert(node_id, abs_rect);
        self.clip_rects.insert(node_id, clipped);

        let node_inner = Rect::new(
            abs_rect.x + info.border_left,
            abs_rect.y + info.border_top,
            abs_rect.width - info.border_left - info.border_right,
            abs_rect.height - info.border_top - info.border_bottom,
        );

        let child_clip = match (info.clip_x, info.clip_y) {
            (true, true) => node_inner.intersect(&parent_clip),
            (true, false) => node_inner.intersect_x(&parent_clip),
            (false, true) => node_inner.intersect_y(&parent_clip),
            (false, false) => parent_clip,
        };
        for child_id in tree.children(node_id) {
            self.compute_absolute_rects(
                tree,
                child_id,
                node_inner.position(),
                child_clip,
                info_fn,
            )?;
        }

        Ok(())
    }

    fn taffy_node(&self, id: NodeId) -> LayoutResult<TaffyId> {
        self.nodes
            .get(id)
            .ok_or(LayoutError::NodeNotFound(id))
            .copied()
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
