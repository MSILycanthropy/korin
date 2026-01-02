use ginyu_force::Pose;
use indextree::NodeId;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{document::Document, element::Element};

pub struct BuildContext<'a> {
    pub(crate) document: &'a mut Document,
}

impl<'a> BuildContext<'a> {
    pub const fn new(document: &'a mut Document) -> Self {
        Self { document }
    }

    #[must_use] 
    pub const fn document(&self) -> &Document {
        self.document
    }

    pub const fn document_mut(&mut self) -> &mut Document {
        self.document
    }

    pub fn create_element(&mut self, tag: Pose) -> NodeId {
        self.document.create_element(tag)
    }

    pub fn create_element_with(&mut self, element: Element) -> NodeId {
        self.document.create_element_with(element)
    }

    pub fn create_text(&mut self, content: impl Into<String>) -> NodeId {
        self.document.create_text(content)
    }

    pub fn set_attribute(&mut self, node: NodeId, name: Pose, value: impl Into<String>) {
        if let Some(element) = self.document.get_mut(node).and_then(|n| n.as_element_mut()) {
            element.set_attribute(name, value);
        }
    }

    pub fn set_text(&mut self, node: NodeId, content: impl Into<String>) {
        if let Some(text) = self.document.get_mut(node).and_then(|n| n.as_text_mut()) {
            *text = content.into();
        }
    }

    pub fn add_class(&mut self, node: NodeId, class: Pose) {
        if let Some(element) = self.document.get_mut(node).and_then(|n| n.as_element_mut()) {
            element.add_class(class);
        }
    }

    pub fn set_id(&mut self, node: NodeId, id: Pose) {
        if let Some(element) = self.document.get_mut(node).and_then(|n| n.as_element_mut()) {
            element.set_id(Some(id));
        }
    }

    pub fn create_marker(&mut self) -> NodeId {
        self.document.create_marker()
    }
}

pub struct RebuildContext<'a> {
    pub(crate) document: &'a mut Document,
}

impl<'a> RebuildContext<'a> {
    pub const fn new(document: &'a mut Document) -> Self {
        Self { document }
    }

    #[must_use] 
    pub const fn document(&self) -> &Document {
        self.document
    }

    pub const fn document_mut(&mut self) -> &mut Document {
        self.document
    }

    pub fn set_classes(&mut self, node: NodeId, classes: SmallVec<[Pose; 4]>) {
        if let Some(element) = self.document.get_mut(node).and_then(|n| n.as_element_mut()) {
            element.set_classes(classes);
        }
    }

    pub fn set_attributes(&mut self, node: NodeId, attributes: FxHashMap<Pose, String>) {
        if let Some(element) = self.document.get_mut(node).and_then(|n| n.as_element_mut()) {
            element.set_attributes(attributes);
        }
    }

    pub fn set_attribute(&mut self, node: NodeId, name: Pose, value: impl Into<String>) {
        if let Some(element) = self.document.get_mut(node).and_then(|n| n.as_element_mut()) {
            element.set_attribute(name, value);
        }
    }

    pub fn set_text(&mut self, node: NodeId, content: impl Into<String>) {
        if let Some(text) = self.document.get_mut(node).and_then(|n| n.as_text_mut()) {
            *text = content.into();
        }
    }

    pub fn remove_class(&mut self, node: NodeId, class: Pose) {
        if let Some(element) = self.document.get_mut(node).and_then(|n| n.as_element_mut()) {
            element.remove_class(class);
        }
    }

    pub fn set_id(&mut self, node: NodeId, id: Option<Pose>) {
        if let Some(element) = self.document.get_mut(node).and_then(|n| n.as_element_mut()) {
            element.set_id(id);
        }
    }
}
