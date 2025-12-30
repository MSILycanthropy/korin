use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use ginyu_force::{Pose, pose};

use crate::{Bulma, ComputedStyle, CustomPropertiesMap, ElementState, Layout};

pub trait CapsuleDocument {
    type Element: CapsuleElement;
    type Node: CapsuleNode;
    type NodeId: Copy + Eq;

    fn root(&self) -> Self::NodeId;
    fn get_element(&self, node: Self::NodeId) -> Option<Self::Element>;
    fn get_element_strict(&self, node: Self::NodeId) -> Self::Element {
        self.get_element(node).expect("not a valid element id")
    }
    fn get_node(&self, node: Self::NodeId) -> &Self::Node;
    fn get_node_mut(&mut self, node: Self::NodeId) -> &mut Self::Node;
    fn parent(&self, node: Self::NodeId) -> Option<Self::NodeId>;
    fn children(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;
    fn next_siblings(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId>;
    fn computed_style(&self, node: Self::NodeId) -> Option<&ComputedStyle>;
    fn custom_properties(&self, node: Self::NodeId) -> Option<&CustomPropertiesMap>;
    fn set_style(
        &mut self,
        node: Self::NodeId,
        style: ComputedStyle,
        custom_properties: CustomPropertiesMap,
    );
    fn take_stylist(&mut self) -> Bulma;
    fn set_stylist(&mut self, stylist: Bulma);
}

pub trait CapsuleElement: Sized + Clone + Debug + PartialEq {
    fn tag_name(&self) -> Pose;
    fn id(&self) -> Option<Pose>;
    fn has_class(&self, name: &str) -> bool;
    fn each_class<F: FnMut(Pose)>(&self, callback: F);
    fn get_attribute(&self, name: Pose) -> Option<&str>;
    fn style_attribute(&self) -> Option<&str> {
        self.get_attribute(pose!("style"))
    }
    fn state(&self) -> ElementState;
    fn parent(&self) -> Option<Self>;
    fn prev_sibling(&self) -> Option<Self>;
    fn next_sibling(&self) -> Option<Self>;
    fn has_children(&self) -> bool;
    fn is_first_child(&self) -> bool {
        self.prev_sibling().is_none()
    }
    fn is_last_child(&self) -> bool {
        self.next_sibling().is_none()
    }
    fn sibling_index(&self) -> usize {
        let mut index = 1;
        let mut current = self.clone();
        while let Some(prev) = current.prev_sibling() {
            index += 1;
            current = prev;
        }
        index
    }
}

pub trait CapsuleNode {
    fn computed_style(&self) -> Option<&ComputedStyle>;
    fn custom_properties(&self) -> Option<&CustomPropertiesMap>;
    fn set_style(&mut self, style: ComputedStyle, custom_properties: CustomPropertiesMap);

    fn layout(&self) -> Layout;
    fn set_layout(&mut self, layout: Layout);
    fn needs_layout(&self) -> bool;
    fn mark_needs_layout(&mut self);
    fn clear_needs_layout(&mut self);

    fn text_content(&self) -> Option<&str>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ConcreteCapsuleElement<E>(pub E);

impl<E> ConcreteCapsuleElement<E> {
    pub const fn new(element: E) -> Self {
        Self(element)
    }

    pub fn into_inner(self) -> E {
        self.0
    }
}

impl<E> Deref for ConcreteCapsuleElement<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for ConcreteCapsuleElement<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
