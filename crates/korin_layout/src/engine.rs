use std::ops::{Deref, DerefMut};

use korin_geometry::Point;
use korin_tree::{NodeId, Tree};
use slotmap::SecondaryMap;
use taffy::{NodeId as TaffyId, TaffyTree};

use crate::{Layout, LayoutResult, Rect, Size, error::LayoutError, measure::taffy_measure};

#[derive(Default, Clone)]
pub struct NodeMeasure {
    pub text: Option<String>,
    pub wrap: bool,
}

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
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub scrollbar_width: f32,
    pub scrollbar_height: f32,
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
        let ctx = NodeMeasure {
            text: None,
            wrap: true,
        };
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
        wrap: bool,
    ) -> LayoutResult<TaffyId> {
        let text_len = text.len();
        let ctx = NodeMeasure {
            text: Some(text.into()),
            wrap,
        };
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

    pub fn update_text(&mut self, id: NodeId, text: String, wrap: bool) -> LayoutResult<()> {
        if let Some(&taffy_id) = self.nodes.get(id) {
            let log_text = text.clone();
            self.taffy.set_node_context(
                taffy_id,
                Some(NodeMeasure {
                    text: Some(text),
                    wrap,
                }),
            )?;
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
    pub fn content_size(&self, id: NodeId) -> Option<Size<f32>> {
        let taffy_id = self.nodes.get(id)?;
        let layout = self.taffy.layout(*taffy_id).ok()?;

        Some(Size::new(
            layout.content_size.width,
            layout.content_size.height,
        ))
    }

    pub fn hit_test<T>(
        &self,
        tree: &Tree<T>,
        point: Point<f32>,
        z_index: impl Fn(&T) -> i32,
    ) -> Option<NodeId> {
        let root = tree.root()?;

        self.hit_test_node(tree, root, point, &z_index)
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
            abs_rect.width - info.border_left - info.border_right - info.scrollbar_width,
            abs_rect.height - info.border_top - info.border_bottom - info.scrollbar_height,
        );

        let child_clip = match (info.clip_x, info.clip_y) {
            (true, true) => node_inner.intersect(&parent_clip),
            (true, false) => node_inner.intersect_x(&parent_clip),
            (false, true) => node_inner.intersect_y(&parent_clip),
            (false, false) => parent_clip,
        };

        let child_origin = Point::new(node_inner.x - info.scroll_x, node_inner.y - info.scroll_y);

        for child_id in tree.children(node_id) {
            self.compute_absolute_rects(tree, child_id, child_origin, child_clip, info_fn)?;
        }

        Ok(())
    }

    fn hit_test_node<T>(
        &self,
        tree: &Tree<T>,
        node_id: NodeId,
        point: Point<f32>,
        z_index: &impl Fn(&T) -> i32,
    ) -> Option<NodeId> {
        let clip = self.clip_rect(node_id)?;

        if !clip.contains(point) {
            return None;
        }

        let mut children = tree.children(node_id);
        children.sort_by_key(|&id| tree.get(id).map_or(0, |n| -z_index(n)));

        for child_id in children {
            if let Some(hit) = self.hit_test_node(tree, child_id, point, z_index) {
                return Some(hit);
            }
        }

        let rect = self.absolute_rect(node_id)?;
        if rect.contains(point) {
            Some(node_id)
        } else {
            None
        }
    }

    #[must_use]
    pub fn max_scroll(&self, id: NodeId) -> Option<Point<f32>> {
        let taffy_id = self.nodes.get(id)?;
        let layout = self.taffy.layout(*taffy_id).ok()?;

        let max_x = (layout.content_size.width - layout.size.width).max(0.0);
        let max_y = (layout.content_size.height - layout.size.height).max(0.0);

        Some(Point::new(max_x, max_y))
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

#[cfg(test)]
mod tests {
    use super::*;
    use korin_tree::Tree;
    use taffy::Position;

    fn default_info() -> LayoutInfo {
        LayoutInfo {
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            clip_x: false,
            clip_y: false,
            scroll_x: 0.0,
            scroll_y: 0.0,
            scrollbar_width: 0.0,
            scrollbar_height: 0.0,
        }
    }

    fn bordered_info() -> LayoutInfo {
        LayoutInfo {
            border_left: 1.0,
            border_top: 1.0,
            border_right: 1.0,
            border_bottom: 1.0,
            clip_x: false,
            clip_y: false,
            scroll_x: 0.0,
            scroll_y: 0.0,
            scrollbar_width: 0.0,
            scrollbar_height: 0.0,
        }
    }

    fn clipping_info() -> LayoutInfo {
        LayoutInfo {
            border_left: 0.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            clip_x: true,
            clip_y: true,
            scroll_x: 0.0,
            scroll_y: 0.0,
            scrollbar_width: 0.0,
            scrollbar_height: 0.0,
        }
    }

    #[test]
    fn compute_absolute_rects_single_node() {
        let mut tree: Tree<i32> = Tree::new();
        let mut engine = Engine::new();

        let root = tree.new_leaf(0);
        tree.set_root(root).expect("failed");
        engine
            .insert(Layout::new().w(100).h(50), root)
            .expect("failed");

        engine
            .compute(&tree, Size::new(100.0, 50.0), |_| default_info())
            .expect("failed");

        let rect = engine.absolute_rect(root).expect("failed");
        assert!((rect.x - 0.0).abs() < f32::EPSILON);
        assert!((rect.y - 0.0).abs() < f32::EPSILON);
        assert!((rect.width - 100.0).abs() < f32::EPSILON);
        assert!((rect.height - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn compute_absolute_rects_nested() {
        let mut tree: Tree<i32> = Tree::new();
        let mut engine = Engine::new();

        let root = tree.new_leaf(0);
        let child = tree.new_leaf(0);
        tree.set_root(root).expect("failed");
        tree.append(root, child).expect("failed");

        engine
            .insert(Layout::new().w(100).h(100).p(10), root)
            .expect("failed");
        engine
            .insert(Layout::new().w(50).h(50), child)
            .expect("failed");
        engine.append(root, child).expect("failed");

        engine
            .compute(&tree, Size::new(100.0, 100.0), |_| default_info())
            .expect("failed");

        let root_rect = engine.absolute_rect(root).expect("failed");
        assert!((root_rect.x - 0.0).abs() < f32::EPSILON);
        assert!((root_rect.y - 0.0).abs() < f32::EPSILON);

        let child_rect = engine.absolute_rect(child).expect("failed");
        assert!((child_rect.x - 10.0).abs() < f32::EPSILON);
        assert!((child_rect.y - 10.0).abs() < f32::EPSILON);
        assert!((child_rect.width - 50.0).abs() < f32::EPSILON);
        assert!((child_rect.height - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clip_rects_with_borders() {
        let mut tree: Tree<i32> = Tree::new();
        let mut engine = Engine::new();

        let root = tree.new_leaf(0);
        let child = tree.new_leaf(0);
        tree.set_root(root).expect("failed");
        tree.append(root, child).expect("failed");

        engine
            .insert(Layout::new().w(100).h(100), root)
            .expect("failed");
        engine
            .insert(Layout::new().w(50).h(50), child)
            .expect("failed");
        engine.append(root, child).expect("failed");

        engine
            .compute(&tree, Size::new(100.0, 100.0), |_| bordered_info())
            .expect("failed");

        let child_rect = engine.absolute_rect(child).expect("failed");

        assert!((child_rect.x - 1.0).abs() < f32::EPSILON);
        assert!((child_rect.y - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clip_rects_with_overflow() {
        let mut tree: Tree<i32> = Tree::new();
        let mut engine = Engine::new();

        let root = tree.new_leaf(0);
        let child = tree.new_leaf(0);
        tree.set_root(root).expect("failed");
        tree.append(root, child).expect("failed");

        engine
            .insert(Layout::new().w(50).h(50), root)
            .expect("failed");
        engine
            .insert(Layout::new().w(100).h(100), child)
            .expect("failed");
        engine.append(root, child).expect("failed");

        engine
            .compute(&tree, Size::new(50.0, 50.0), |_| clipping_info())
            .expect("failed");

        let child_clip = engine.clip_rect(child).expect("failed");

        assert!((child_clip.width - 50.0).abs() < f32::EPSILON);
        assert!((child_clip.height - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn hit_test_single_node() {
        let mut tree: Tree<i32> = Tree::new();
        let mut engine = Engine::new();

        let root = tree.new_leaf(0);
        tree.set_root(root).expect("failed");
        engine
            .insert(Layout::new().w(100).h(100), root)
            .expect("failed");

        engine
            .compute(&tree, Size::new(100.0, 100.0), |_| default_info())
            .expect("failed");

        let hit = engine.hit_test(&tree, Point::new(50.0, 50.0), |_| 0);
        assert_eq!(hit, Some(root));

        let miss = engine.hit_test(&tree, Point::new(150.0, 50.0), |_| 0);
        assert_eq!(miss, None);
    }

    #[test]
    fn hit_test_nested_returns_deepest() {
        let mut tree: Tree<i32> = Tree::new();
        let mut engine = Engine::new();

        let root = tree.new_leaf(0);
        let child = tree.new_leaf(0);
        tree.set_root(root).expect("failed");
        tree.append(root, child).expect("failed");

        engine
            .insert(Layout::new().w(100).h(100), root)
            .expect("failed");
        engine
            .insert(Layout::new().w(50).h(50), child)
            .expect("failed");
        engine.append(root, child).expect("failed");

        engine
            .compute(&tree, Size::new(100.0, 100.0), |_| default_info())
            .expect("failed");

        let hit_child = engine.hit_test(&tree, Point::new(25.0, 25.0), |_| 0);
        assert_eq!(hit_child, Some(child));

        let hit_root = engine.hit_test(&tree, Point::new(75.0, 75.0), |_| 0);
        assert_eq!(hit_root, Some(root));
    }

    #[test]
    fn hit_test_respects_z_index() {
        let mut tree: Tree<i32> = Tree::new();
        let mut engine = Engine::new();

        let root = tree.new_leaf(0);
        let child_low = tree.new_leaf(1); // z-index 1
        let child_high = tree.new_leaf(10); // z-index 10
        tree.set_root(root).expect("failed");
        tree.append(root, child_low).expect("failed");
        tree.append(root, child_high).expect("failed");

        engine
            .insert(
                Layout::new().w(100).h(100).position(Position::Relative),
                root,
            )
            .expect("failed");
        engine
            .insert(
                Layout::new()
                    .w(50)
                    .h(50)
                    .position(Position::Absolute)
                    .top(0)
                    .left(0),
                child_low,
            )
            .expect("failed");
        engine
            .insert(
                Layout::new()
                    .w(50)
                    .h(50)
                    .position(Position::Absolute)
                    .top(0)
                    .left(0),
                child_high,
            )
            .expect("failed");
        engine.append(root, child_low).expect("failed");
        engine.append(root, child_high).expect("failed");
        engine
            .compute(&tree, Size::new(100.0, 100.0), |_| default_info())
            .expect("failed");

        let hit = engine.hit_test(&tree, Point::new(25.0, 25.0), |&z| z);
        assert_eq!(hit, Some(child_high));
    }

    #[test]
    fn hit_test_respects_clip() {
        let mut tree: Tree<i32> = Tree::new();
        let mut engine = Engine::new();

        let root = tree.new_leaf(0);
        let child = tree.new_leaf(0);
        tree.set_root(root).expect("failed");
        tree.append(root, child).expect("failed");

        engine
            .insert(Layout::new().w(50).h(50), root)
            .expect("failed");
        engine
            .insert(Layout::new().w(100).h(100), child)
            .expect("failed");
        engine.append(root, child).expect("failed");

        engine
            .compute(&tree, Size::new(50.0, 50.0), |_| clipping_info())
            .expect("failed");

        // Inside clipped area - hits child
        let hit_inside = engine.hit_test(&tree, Point::new(25.0, 25.0), |_| 0);
        assert_eq!(hit_inside, Some(child));

        // Outside clip (but inside child's actual rect) - misses
        let hit_outside = engine.hit_test(&tree, Point::new(75.0, 75.0), |_| 0);
        assert_eq!(hit_outside, None);
    }
}
