use capsule_corp::{ComputedStyle, CustomPropertiesMap, Layout};

use crate::element::Element;

#[derive(Debug, PartialEq)]
pub struct Node {
    pub data: NodeData,
    pub style: Option<ComputedStyle>,
    pub custom_properties: Option<CustomPropertiesMap>,
    pub layout: Layout,
    pub needs_layout: bool,
}

impl Node {
    #[must_use]
    pub const fn root() -> Self {
        Self {
            data: NodeData::Root,
            style: None,
            custom_properties: None,
            layout: Layout::ZERO,
            needs_layout: true,
        }
    }

    #[must_use]
    pub fn element(element: Element) -> Self {
        Self {
            data: NodeData::Element(element),
            style: Some(ComputedStyle::default()),
            custom_properties: None,
            layout: Layout::ZERO,
            needs_layout: true,
        }
    }

    pub fn text(content: impl Into<String>) -> Self {
        let content = content.into();

        Self {
            data: NodeData::Text(content),
            style: None,
            custom_properties: None,
            layout: Layout::ZERO,
            needs_layout: true,
        }
    }

    #[must_use]
    pub const fn marker() -> Self {
        Self {
            data: NodeData::Marker,
            style: None,
            custom_properties: None,
            layout: Layout::ZERO,
            needs_layout: false,
        }
    }

    #[must_use]
    pub const fn as_element(&self) -> Option<&Element> {
        match &self.data {
            NodeData::Element(element) => Some(element),
            _ => None,
        }
    }

    pub const fn as_element_mut(&mut self) -> Option<&mut Element> {
        match &mut self.data {
            NodeData::Element(element) => Some(element),
            _ => None,
        }
    }
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match &self.data {
            NodeData::Text(s) => Some(s),
            _ => None,
        }
    }

    pub const fn as_text_mut(&mut self) -> Option<&mut String> {
        match &mut self.data {
            NodeData::Text(s) => Some(s),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_root(&self) -> bool {
        matches!(self.data, NodeData::Root)
    }

    #[must_use]
    pub const fn is_element(&self) -> bool {
        matches!(self.data, NodeData::Element(_))
    }

    #[must_use]
    pub const fn is_text(&self) -> bool {
        matches!(self.data, NodeData::Text(_))
    }

    #[must_use]
    pub const fn is_marker(&self) -> bool {
        matches!(self.data, NodeData::Marker)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeData {
    Root,
    Element(Element),
    Text(String),

    /// Anonymous marker node for control flow (Show, For, etc.)
    /// Invisible and skipped during layout/render.
    Marker,
}
