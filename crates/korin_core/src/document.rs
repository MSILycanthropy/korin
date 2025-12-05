use korin_layout::{Rect, Size};
use korin_tree::{Node as TreeNode, NodeId, Tree};
use ratatui::Frame;
use taffy::{NodeId as TaffyId, TaffyTree};

use crate::{
    element::Element,
    error::{KorinError, KorinResult},
};

pub struct Node<'a> {
    pub element: Element<'a>,
    pub taffy_id: TaffyId,
    pub rect: Rect,
}

impl<'a> Node<'a> {
    #[must_use]
    pub fn new(element: Element<'a>, taffy_id: TaffyId) -> Self {
        Self {
            element,
            taffy_id,
            rect: Rect::default(),
        }
    }
}

pub struct Document<'a> {
    tree: Tree<Node<'a>>,
    taffy: TaffyTree<()>,
}

impl<'a> Document<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tree: Tree::new(),
            taffy: TaffyTree::new(),
        }
    }

    /// Sets the root of this [`Document`].
    ///
    /// # Errors
    ///
    /// This function will return an error if creating the `taffy` Node fails.
    pub fn set_root(&mut self, element: Element<'a>) -> KorinResult<NodeId> {
        let taffy_id = self.create_taffy_node(&element, &[])?;

        let node_id = self.tree.set_root(Node::new(element, taffy_id));

        Ok(node_id)
    }

    /// Append a child to a node in the tree
    ///
    /// # Errors
    ///
    /// This function will return an error if
    ///     - Creating a `taffy` Node fails
    ///     - The `parent` node is not in the tree
    ///     - Adding a child to the `parent` node fails
    pub fn append(&mut self, parent: NodeId, element: Element<'a>) -> KorinResult<NodeId> {
        let taffy_id = self.create_taffy_node(&element, &[])?;

        let Some(node_id) = self.tree.append(parent, Node::new(element, taffy_id)) else {
            return Err(KorinError::NodeNotFound(parent));
        };

        let parent = self.get_node(parent)?;

        self.taffy.add_child(parent.data.taffy_id, taffy_id)?;

        Ok(node_id)
    }

    fn create_taffy_node(
        &mut self,
        element: &Element<'a>,
        children: &[TaffyId],
    ) -> KorinResult<TaffyId> {
        let style = match element {
            Element::Div(div) => div.layout.0.clone(),
            Element::Text(_) => taffy::Style::default(),
        };

        let id = self.taffy.new_with_children(style, children)?;

        Ok(id)
    }

    /// Run the layout pass for this document
    ///
    /// # Errors
    ///
    /// This function will return an error if
    ///     - There is no root for the document
    ///     - Computing the layout fails at the taffy level
    ///     - Applying the layout to the tree fails
    pub fn layout(&mut self, size: Size) -> KorinResult<()> {
        let root_id = self.root_id()?;
        let root = self.get_node(root_id)?;

        self.taffy.compute_layout(root.data.taffy_id, size.into())?;
        self.apply_layout(root_id, 0, 0)?;

        Ok(())
    }

    fn apply_layout(&mut self, node_id: NodeId, offset_x: u16, offset_y: u16) -> KorinResult<()> {
        let node = self.get_node(node_id)?;
        let taffy_id = node.data.taffy_id;

        let layout = self.taffy.layout(taffy_id)?;

        let rect: Rect = layout.into();

        let absolute_rect = Rect::new(
            rect.x + offset_x,
            rect.y + offset_y,
            rect.width,
            rect.height,
        );

        if let Some(node) = self.tree.get_mut(node_id) {
            node.data.rect = absolute_rect;
        } else {
            return Err(KorinError::NodeNotFound(node_id));
        }

        let children: Vec<NodeId> = self
            .tree
            .get(node_id)
            .map(|n| n.children.clone())
            .unwrap_or_default();

        for child_id in children {
            self.apply_layout(child_id, absolute_rect.x, absolute_rect.y)?;
        }

        Ok(())
    }

    /// The render pass for a ['Document]
    ///
    /// # Errors
    ///
    /// This function will return an error if the node to render is not in the tree.
    pub fn render(&self, frame: &mut Frame) -> KorinResult<()> {
        let root_id = self.root_id()?;

        self.render_node(frame, root_id)
    }

    fn render_node(&self, frame: &mut Frame, node_id: NodeId) -> KorinResult<()> {
        let node = self.get_node(node_id)?;

        let area: ratatui::layout::Rect = node.data.rect.into();

        frame.render_widget(&node.data.element, area);

        for &child_id in &node.children {
            self.render_node(frame, child_id)?;
        }

        Ok(())
    }

    fn root_id(&self) -> KorinResult<NodeId> {
        self.tree.root().ok_or(KorinError::NoRoot)
    }

    fn get_node(&self, node_id: NodeId) -> KorinResult<&TreeNode<Node<'a>>> {
        self.tree
            .get(node_id)
            .ok_or(KorinError::NodeNotFound(node_id))
    }
}

impl Default for Document<'_> {
    fn default() -> Self {
        Self::new()
    }
}
