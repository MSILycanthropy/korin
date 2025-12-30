//! End-to-end integration tests for the Brief layout system.

use std::collections::HashMap;

use capsule_corp::{
    AlignContent, AlignItems, AlignSelf, BorderStyle, Bulma, CapsuleDocument, CapsuleElement,
    CapsuleNode, ComputedStyle, CustomPropertiesMap, Dimension, Display, Edges, ElementState,
    FlexDirection, FlexWrap, JustifyContent, Layout, Length, Point, Size, compute_layout,
};
use ginyu_force::Pose;

// --- Mock Types ---

#[derive(Debug, Clone)]
struct MockElementData {
    tag: Pose,
    id: Option<Pose>,
    classes: Vec<Pose>,
    state: ElementState,
}

#[derive(Debug, Clone)]
struct MockNode {
    layout: Layout,
    needs_layout: bool,
    kind: MockNodeKind,
}

#[derive(Debug, Clone)]
enum MockNodeKind {
    Element(Box<MockElementNode>),
    Text(String),
}

#[derive(Debug, Clone)]
struct MockElementNode {
    tag: Pose,
    id: Option<Pose>,
    classes: Vec<Pose>,
    attributes: HashMap<Pose, String>,
    state: ElementState,
    inline_style: Option<String>,
    parent_data: Option<Box<MockElementData>>,
    prev_sibling_data: Option<Box<MockElementData>>,
    next_sibling_data: Option<Box<MockElementData>>,
    has_children: bool,
    style: Option<ComputedStyle>,
    custom_properties: Option<CustomPropertiesMap>,
}

impl MockNode {
    fn element(tag: &str) -> Self {
        Self {
            layout: Layout::ZERO,
            needs_layout: true,
            kind: MockNodeKind::Element(
                MockElementNode {
                    tag: Pose::from(tag),
                    id: None,
                    classes: vec![],
                    attributes: HashMap::new(),
                    state: ElementState::empty(),
                    inline_style: None,
                    parent_data: None,
                    prev_sibling_data: None,
                    next_sibling_data: None,
                    has_children: false,
                    style: None,
                    custom_properties: None,
                }
                .into(),
            ),
        }
    }

    fn text(content: &str) -> Self {
        Self {
            layout: Layout::ZERO,
            needs_layout: true,
            kind: MockNodeKind::Text(content.to_string()),
        }
    }

    fn as_element(&self) -> Option<MockElement> {
        match &self.kind {
            MockNodeKind::Element(el) => Some(MockElement {
                tag: el.tag,
                id: el.id,
                classes: el.classes.clone(),
                attributes: el.attributes.clone(),
                state: el.state,
                inline_style: el.inline_style.clone(),
                parent_data: el.parent_data.clone(),
                prev_sibling_data: el.prev_sibling_data.clone(),
                next_sibling_data: el.next_sibling_data.clone(),
                has_children: el.has_children,
            }),
            MockNodeKind::Text(_) => None,
        }
    }

    const fn as_element_node(&self) -> Option<&MockElementNode> {
        match &self.kind {
            MockNodeKind::Element(el) => Some(el),
            MockNodeKind::Text(_) => None,
        }
    }

    const fn as_element_node_mut(&mut self) -> Option<&mut MockElementNode> {
        match &mut self.kind {
            MockNodeKind::Element(el) => Some(el),
            MockNodeKind::Text(_) => None,
        }
    }
}

impl CapsuleNode for MockNode {
    fn computed_style(&self) -> Option<&ComputedStyle> {
        match &self.kind {
            MockNodeKind::Element(el) => el.style.as_ref(),
            MockNodeKind::Text(_) => None,
        }
    }

    fn custom_properties(&self) -> Option<&CustomPropertiesMap> {
        match &self.kind {
            MockNodeKind::Element(el) => el.custom_properties.as_ref(),
            MockNodeKind::Text(_) => None,
        }
    }

    fn set_style(&mut self, style: ComputedStyle, custom_properties: CustomPropertiesMap) {
        if let MockNodeKind::Element(el) = &mut self.kind {
            el.style = Some(style);
            el.custom_properties = Some(custom_properties);
        }
    }

    fn layout(&self) -> Layout {
        self.layout
    }

    fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
    }

    fn needs_layout(&self) -> bool {
        self.needs_layout
    }

    fn mark_needs_layout(&mut self) {
        self.needs_layout = true;
    }

    fn clear_needs_layout(&mut self) {
        self.needs_layout = false;
    }

    fn text_content(&self) -> Option<&str> {
        match &self.kind {
            MockNodeKind::Text(s) => Some(s),
            MockNodeKind::Element(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
struct MockElement {
    tag: Pose,
    id: Option<Pose>,
    classes: Vec<Pose>,
    attributes: HashMap<Pose, String>,
    state: ElementState,
    inline_style: Option<String>,
    parent_data: Option<Box<MockElementData>>,
    prev_sibling_data: Option<Box<MockElementData>>,
    next_sibling_data: Option<Box<MockElementData>>,
    has_children: bool,
}

impl PartialEq for MockElement {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag && self.id == other.id
    }
}

impl CapsuleElement for MockElement {
    fn tag_name(&self) -> Pose {
        self.tag
    }

    fn id(&self) -> Option<Pose> {
        self.id
    }

    fn has_class(&self, name: &str) -> bool {
        self.classes.iter().any(|c| c.as_str() == name)
    }

    fn each_class<F: FnMut(Pose)>(&self, mut f: F) {
        for class in &self.classes {
            f(*class);
        }
    }

    fn get_attribute(&self, name: Pose) -> Option<&str> {
        self.attributes.get(&name).map(String::as_str)
    }

    fn style_attribute(&self) -> Option<&str> {
        self.inline_style.as_deref()
    }

    fn state(&self) -> ElementState {
        self.state
    }

    fn parent(&self) -> Option<Self> {
        self.parent_data.as_ref().map(|data| Self {
            tag: data.tag,
            id: data.id,
            classes: data.classes.clone(),
            state: data.state,
            attributes: HashMap::new(),
            inline_style: None,
            parent_data: None,
            prev_sibling_data: None,
            next_sibling_data: None,
            has_children: false,
        })
    }

    fn prev_sibling(&self) -> Option<Self> {
        self.prev_sibling_data.as_ref().map(|data| Self {
            tag: data.tag,
            id: data.id,
            classes: data.classes.clone(),
            state: data.state,
            attributes: HashMap::new(),
            inline_style: None,
            parent_data: None,
            prev_sibling_data: None,
            next_sibling_data: None,
            has_children: false,
        })
    }

    fn next_sibling(&self) -> Option<Self> {
        self.next_sibling_data.as_ref().map(|data| Self {
            tag: data.tag,
            id: data.id,
            classes: data.classes.clone(),
            state: data.state,
            attributes: HashMap::new(),
            inline_style: None,
            parent_data: None,
            prev_sibling_data: None,
            next_sibling_data: None,
            has_children: false,
        })
    }

    fn has_children(&self) -> bool {
        self.has_children
    }
}

struct MockDocument {
    nodes: HashMap<usize, MockNode>,
    children: HashMap<usize, Vec<usize>>,
    parent: HashMap<usize, usize>,
    stylist: Bulma,
    root: usize,
    next_id: usize,
}

impl MockDocument {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            children: HashMap::new(),
            parent: HashMap::new(),
            stylist: Bulma::new(),
            root: 0,
            next_id: 0,
        }
    }

    fn create_element(&mut self, tag: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, MockNode::element(tag));
        if id == 0 {
            self.root = id;
        }
        id
    }

    fn create_text(&mut self, content: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, MockNode::text(content));
        id
    }

    fn append_child(&mut self, parent: usize, child: usize) {
        if let Some(parent_node) = self.nodes.get_mut(&parent)
            && let Some(el) = parent_node.as_element_node_mut()
        {
            el.has_children = true;
        }

        if let Some(parent_node) = self.nodes.get(&parent)
            && let Some(parent_el) = parent_node.as_element_node()
        {
            let parent_data = MockElementData {
                tag: parent_el.tag,
                id: parent_el.id,
                classes: parent_el.classes.clone(),
                state: parent_el.state,
            };
            if let Some(child_node) = self.nodes.get_mut(&child)
                && let Some(child_el) = child_node.as_element_node_mut()
            {
                child_el.parent_data = Some(Box::new(parent_data));
            }
        }

        let siblings = self.children.entry(parent).or_default();
        if let Some(&prev_id) = siblings.last() {
            if let Some(prev_node) = self.nodes.get(&prev_id)
                && let Some(prev_el) = prev_node.as_element_node()
            {
                let prev_data = MockElementData {
                    tag: prev_el.tag,
                    id: prev_el.id,
                    classes: prev_el.classes.clone(),
                    state: prev_el.state,
                };
                if let Some(child_node) = self.nodes.get_mut(&child)
                    && let Some(child_el) = child_node.as_element_node_mut()
                {
                    child_el.prev_sibling_data = Some(Box::new(prev_data));
                }
            }

            if let Some(child_node) = self.nodes.get(&child)
                && let Some(child_el) = child_node.as_element_node()
            {
                let next_data = MockElementData {
                    tag: child_el.tag,
                    id: child_el.id,
                    classes: child_el.classes.clone(),
                    state: child_el.state,
                };
                if let Some(prev_node) = self.nodes.get_mut(&prev_id)
                    && let Some(prev_el) = prev_node.as_element_node_mut()
                {
                    prev_el.next_sibling_data = Some(Box::new(next_data));
                }
            }
        }

        siblings.push(child);
        self.parent.insert(child, parent);
    }

    fn set_style(&mut self, node: usize, style: ComputedStyle) {
        if let Some(n) = self.nodes.get_mut(&node)
            && let Some(el) = n.as_element_node_mut()
        {
            el.style = Some(style);
        }
    }

    fn layout(&self, node: usize) -> Layout {
        self.nodes.get(&node).map_or(Layout::ZERO, |n| n.layout)
    }
}

impl CapsuleDocument for MockDocument {
    type Element = MockElement;
    type Node = MockNode;
    type NodeId = usize;

    fn root(&self) -> usize {
        self.root
    }

    fn get_node(&self, node: usize) -> &MockNode {
        self.nodes.get(&node).expect("node not found")
    }

    fn get_node_mut(&mut self, node: usize) -> &mut MockNode {
        self.nodes.get_mut(&node).expect("node not found")
    }

    fn get_element(&self, node: usize) -> Option<MockElement> {
        self.nodes.get(&node).and_then(MockNode::as_element)
    }

    fn parent(&self, node: usize) -> Option<usize> {
        self.parent.get(&node).copied()
    }

    fn children(&self, node: usize) -> impl Iterator<Item = usize> {
        self.children
            .get(&node)
            .map(|v| v.iter().copied())
            .into_iter()
            .flatten()
    }

    fn next_siblings(&self, node: usize) -> impl Iterator<Item = usize> {
        let parent = self.parent.get(&node).copied();
        let siblings = parent.and_then(|p| self.children.get(&p));

        siblings
            .into_iter()
            .flatten()
            .copied()
            .skip_while(move |&id| id != node)
            .skip(1)
    }

    fn computed_style(&self, node: usize) -> Option<&ComputedStyle> {
        self.nodes
            .get(&node)
            .and_then(|n| n.as_element_node())
            .and_then(|el| el.style.as_ref())
    }

    fn custom_properties(&self, node: usize) -> Option<&CustomPropertiesMap> {
        self.nodes
            .get(&node)
            .and_then(|n| n.as_element_node())
            .and_then(|el| el.custom_properties.as_ref())
    }

    fn set_style(&mut self, node: usize, style: ComputedStyle, cp: CustomPropertiesMap) {
        if let Some(n) = self.nodes.get_mut(&node)
            && let Some(el) = n.as_element_node_mut()
        {
            el.style = Some(style);
            el.custom_properties = Some(cp);
        }
    }

    fn take_stylist(&mut self) -> Bulma {
        std::mem::take(&mut self.stylist)
    }

    fn set_stylist(&mut self, stylist: Bulma) {
        self.stylist = stylist;
    }
}

// =============================================================================
// TEXT NODE TESTS
// =============================================================================

#[test]
fn text_node_measures_content() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let text = doc.create_text("hello");
    doc.append_child(root, text);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    let text_layout = doc.layout(text);
    assert_eq!(text_layout.resolved_box.content_size.width, 5);
    assert_eq!(text_layout.resolved_box.content_size.height, 1);
}

#[test]
fn text_node_in_block_stacks() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let text1 = doc.create_text("first");
    let text2 = doc.create_text("second");
    doc.append_child(root, text1);
    doc.append_child(root, text2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(text1).location, Point::new(0, 0));
    assert_eq!(doc.layout(text2).location, Point::new(0, 1));
}

#[test]
fn text_node_in_flex_row() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let text1 = doc.create_text("hello");
    let text2 = doc.create_text("world");
    doc.append_child(root, text1);
    doc.append_child(root, text2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(text1).location, Point::new(0, 0));
    assert_eq!(doc.layout(text2).location, Point::new(5, 0));
}

#[test]
fn text_node_in_flex_with_gap() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let text1 = doc.create_text("aa");
    let text2 = doc.create_text("bb");
    doc.append_child(root, text1);
    doc.append_child(root, text2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: Length::Cells(3),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(text1).location, Point::new(0, 0));
    assert_eq!(doc.layout(text2).location, Point::new(5, 0));
}

#[test]
fn text_node_mixed_with_elements() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let text = doc.create_text("label");
    let button = doc.create_element("button");
    doc.append_child(root, text);
    doc.append_child(root, button);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
    );
    doc.set_style(
        button,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(10)),
            height: Dimension::Length(Length::Cells(3)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(text).location, Point::new(0, 0));
    assert_eq!(doc.layout(button).location, Point::new(5, 0));
}

#[test]
fn text_node_wraps_in_narrow_container() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let text = doc.create_text("hello world");
    doc.append_child(root, text);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(8, 100));

    let text_layout = doc.layout(text);
    assert_eq!(text_layout.resolved_box.content_size.height, 2);
}

#[test]
fn text_node_doesnt_grow_in_flex() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let text = doc.create_text("hi");
    let spacer = doc.create_element("div");
    doc.append_child(root, text);
    doc.append_child(root, spacer);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
    );
    doc.set_style(
        spacer,
        ComputedStyle {
            flex_grow: 1.0,
            height: Dimension::Length(Length::Cells(1)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(text).resolved_box.content_size.width, 2);
    assert_eq!(doc.layout(spacer).resolved_box.border_box_size().width, 98);
}

// =============================================================================
// BLOCK LAYOUT TESTS
// =============================================================================

#[test]
fn block_stacks_children_vertically() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            display: Display::Block,
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            display: Display::Block,
            height: Dimension::Length(Length::Cells(20)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location, Point::new(0, 0));
    assert_eq!(doc.layout(child2).location, Point::new(0, 10));
}

#[test]
fn block_respects_margins() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            display: Display::Block,
            height: Dimension::Length(Length::Cells(10)),
            margin: Edges::new(
                Length::Cells(5),
                Length::Cells(10),
                Length::Cells(5),
                Length::Cells(10),
            ),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child).location, Point::new(10, 5));
}

// =============================================================================
// INLINE LAYOUT TESTS
// =============================================================================

#[test]
fn inline_flows_horizontally() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("span");
    let child2 = doc.create_element("span");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Inline,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            display: Display::Block,
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            display: Display::Block,
            width: Dimension::Length(Length::Cells(30)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location, Point::new(0, 0));
    assert_eq!(doc.layout(child2).location, Point::new(20, 0));
}

#[test]
fn inline_wraps_when_full() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("span");
    let child2 = doc.create_element("span");
    let child3 = doc.create_element("span");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Inline,
            ..Default::default()
        },
    );
    let child_style = ComputedStyle {
        display: Display::Block,
        width: Dimension::Length(Length::Cells(40)),
        height: Dimension::Length(Length::Cells(10)),
        ..Default::default()
    };
    doc.set_style(child1, child_style.clone());
    doc.set_style(child2, child_style.clone());
    doc.set_style(child3, child_style);

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location, Point::new(0, 0));
    assert_eq!(doc.layout(child2).location, Point::new(40, 0));
    assert_eq!(doc.layout(child3).location, Point::new(0, 10));
}

// =============================================================================
// FLEX DIRECTION TESTS
// =============================================================================

#[test]
fn flex_row_lays_out_horizontally() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(30)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location, Point::new(0, 0));
    assert_eq!(doc.layout(child2).location, Point::new(20, 0));
}

#[test]
fn flex_column_lays_out_vertically() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(15)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location, Point::new(0, 0));
    assert_eq!(doc.layout(child2).location, Point::new(0, 10));
}

#[test]
fn flex_row_reverse() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::RowReverse,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(30)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    // Items reversed: child2 first, then child1
    assert_eq!(doc.layout(child2).location.x, 0);
    assert_eq!(doc.layout(child1).location.x, 30);
}

#[test]
fn flex_column_reverse() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(15)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    // Items reversed: child2 first, then child1
    assert_eq!(doc.layout(child2).location.y, 0);
    assert_eq!(doc.layout(child1).location.y, 15);
}

// =============================================================================
// FLEX SIZING TESTS
// =============================================================================

#[test]
fn flex_grow_distributes_space() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            flex_grow: 1.0,
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            flex_grow: 1.0,
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).resolved_box.border_box_size().width, 50);
    assert_eq!(doc.layout(child2).resolved_box.border_box_size().width, 50);
}

#[test]
fn flex_grow_respects_ratio() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            flex_grow: 1.0,
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            flex_grow: 3.0,
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).resolved_box.border_box_size().width, 25);
    assert_eq!(doc.layout(child2).resolved_box.border_box_size().width, 75);
}

#[test]
fn flex_shrink_removes_overflow() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            flex_shrink: 1.0,
            width: Dimension::Length(Length::Cells(60)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            flex_shrink: 1.0,
            width: Dimension::Length(Length::Cells(60)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).resolved_box.border_box_size().width, 50);
    assert_eq!(doc.layout(child2).resolved_box.border_box_size().width, 50);
}

#[test]
fn flex_basis_sets_initial_size() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            flex_basis: Dimension::Length(Length::Cells(30)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            flex_basis: Dimension::Length(Length::Cells(40)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).resolved_box.border_box_size().width, 30);
    assert_eq!(doc.layout(child2).resolved_box.border_box_size().width, 40);
}

#[test]
fn flex_min_width_constraint() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            flex_shrink: 1.0,
            width: Dimension::Length(Length::Cells(80)),
            min_width: Dimension::Length(Length::Cells(60)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            flex_shrink: 1.0,
            width: Dimension::Length(Length::Cells(80)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    // child1 can't shrink below 60, child2 takes the rest
    assert_eq!(doc.layout(child1).resolved_box.border_box_size().width, 60);
    assert_eq!(doc.layout(child2).resolved_box.border_box_size().width, 40);
}

#[test]
fn flex_max_width_constraint() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            flex_grow: 1.0,
            max_width: Dimension::Length(Length::Cells(30)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            flex_grow: 1.0,
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    // child1 can't grow past 30, child2 gets the rest
    assert_eq!(doc.layout(child1).resolved_box.border_box_size().width, 30);
    assert_eq!(doc.layout(child2).resolved_box.border_box_size().width, 70);
}

// =============================================================================
// FLEX JUSTIFY CONTENT TESTS
// =============================================================================

#[test]
fn flex_justify_content_flex_start() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child).location.x, 0);
}

#[test]
fn flex_justify_content_flex_end() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexEnd,
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child).location.x, 80);
}

#[test]
fn flex_justify_content_center() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child).location.x, 40);
}

#[test]
fn flex_justify_content_space_between() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location.x, 0);
    assert_eq!(doc.layout(child2).location.x, 80);
}

#[test]
fn flex_justify_content_space_around() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceAround,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    // Free space 60, 2 items, 30 per item, 15 on each side
    assert_eq!(doc.layout(child1).location.x, 15);
    assert_eq!(doc.layout(child2).location.x, 65);
}

#[test]
fn flex_justify_content_space_evenly() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceEvenly,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    // Free space 60, 3 slots, 20 each
    assert_eq!(doc.layout(child1).location.x, 20);
    assert_eq!(doc.layout(child2).location.x, 60);
}

// =============================================================================
// FLEX ALIGN ITEMS TESTS
// =============================================================================

#[test]
fn flex_align_items_flex_start() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            height: Dimension::Length(Length::Cells(50)),
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 50));

    assert_eq!(doc.layout(child).location.y, 0);
}

#[test]
fn flex_align_items_flex_end() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexEnd,
            height: Dimension::Length(Length::Cells(50)),
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 50));

    assert_eq!(doc.layout(child).location.y, 40);
}

#[test]
fn flex_align_items_center() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            height: Dimension::Length(Length::Cells(50)),
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 50));

    assert_eq!(doc.layout(child).location.y, 20);
}

#[test]
fn flex_align_items_stretch() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            height: Dimension::Length(Length::Cells(50)),
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 50));

    assert_eq!(doc.layout(child).location.y, 0);
    assert_eq!(doc.layout(child).resolved_box.border_box_size().height, 50);
}

#[test]
fn flex_align_self_override() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            height: Dimension::Length(Length::Cells(50)),
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            align_self: AlignSelf::FlexEnd,
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 50));

    assert_eq!(doc.layout(child1).location.y, 0);
    assert_eq!(doc.layout(child2).location.y, 40);
}

// =============================================================================
// FLEX WRAP TESTS
// =============================================================================

#[test]
fn flex_wrap_nowrap() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    let child3 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::NoWrap,
            ..Default::default()
        },
    );
    let child_style = ComputedStyle {
        width: Dimension::Length(Length::Cells(40)),
        height: Dimension::Length(Length::Cells(10)),
        ..Default::default()
    };
    doc.set_style(child1, child_style.clone());
    doc.set_style(child2, child_style.clone());
    doc.set_style(child3, child_style);

    compute_layout(&mut doc, root, Size::new(100, 100));

    // All on same line even though they overflow
    assert_eq!(doc.layout(child1).location.y, 0);
    assert_eq!(doc.layout(child2).location.y, 0);
    assert_eq!(doc.layout(child3).location.y, 0);
}

#[test]
fn flex_wrap_creates_multiple_lines() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    let child3 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..Default::default()
        },
    );
    let child_style = ComputedStyle {
        width: Dimension::Length(Length::Cells(40)),
        height: Dimension::Length(Length::Cells(10)),
        ..Default::default()
    };
    doc.set_style(child1, child_style.clone());
    doc.set_style(child2, child_style.clone());
    doc.set_style(child3, child_style);

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location, Point::new(0, 0));
    assert_eq!(doc.layout(child2).location, Point::new(40, 0));
    assert_eq!(doc.layout(child3).location, Point::new(0, 10));
}

#[test]
fn flex_wrap_reverse() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    let child3 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::WrapReverse,
            ..Default::default()
        },
    );
    let child_style = ComputedStyle {
        width: Dimension::Length(Length::Cells(40)),
        height: Dimension::Length(Length::Cells(10)),
        ..Default::default()
    };
    doc.set_style(child1, child_style.clone());
    doc.set_style(child2, child_style.clone());
    doc.set_style(child3, child_style);

    compute_layout(&mut doc, root, Size::new(100, 100));

    // Lines are reversed
    assert_eq!(doc.layout(child3).location.y, 0);
    assert_eq!(doc.layout(child1).location.y, 10);
    assert_eq!(doc.layout(child2).location.y, 10);
}

// =============================================================================
// FLEX ALIGN CONTENT TESTS
// =============================================================================

#[test]
fn flex_align_content_flex_start() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    let child3 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::FlexStart,
            height: Dimension::Length(Length::Cells(100)),
            ..Default::default()
        },
    );
    let child_style = ComputedStyle {
        width: Dimension::Length(Length::Cells(40)),
        height: Dimension::Length(Length::Cells(10)),
        ..Default::default()
    };
    doc.set_style(child1, child_style.clone());
    doc.set_style(child2, child_style.clone());
    doc.set_style(child3, child_style);

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location.y, 0);
    assert_eq!(doc.layout(child3).location.y, 10);
}

#[test]
fn flex_align_content_flex_end() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    let child3 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::FlexEnd,
            height: Dimension::Length(Length::Cells(100)),
            ..Default::default()
        },
    );
    let child_style = ComputedStyle {
        width: Dimension::Length(Length::Cells(40)),
        height: Dimension::Length(Length::Cells(10)),
        ..Default::default()
    };
    doc.set_style(child1, child_style.clone());
    doc.set_style(child2, child_style.clone());
    doc.set_style(child3, child_style);

    compute_layout(&mut doc, root, Size::new(100, 100));

    // Lines pushed to end, total lines height = 20
    assert_eq!(doc.layout(child1).location.y, 80);
    assert_eq!(doc.layout(child3).location.y, 90);
}

#[test]
fn flex_align_content_center() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    let child3 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::Center,
            height: Dimension::Length(Length::Cells(100)),
            ..Default::default()
        },
    );
    let child_style = ComputedStyle {
        width: Dimension::Length(Length::Cells(40)),
        height: Dimension::Length(Length::Cells(10)),
        ..Default::default()
    };
    doc.set_style(child1, child_style.clone());
    doc.set_style(child2, child_style.clone());
    doc.set_style(child3, child_style);

    compute_layout(&mut doc, root, Size::new(100, 100));

    // Lines centered, total height 20, free space 80, offset 40
    assert_eq!(doc.layout(child1).location.y, 40);
    assert_eq!(doc.layout(child3).location.y, 50);
}

#[test]
fn flex_align_content_space_between() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    let child3 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::SpaceBetween,
            height: Dimension::Length(Length::Cells(100)),
            ..Default::default()
        },
    );
    let child_style = ComputedStyle {
        width: Dimension::Length(Length::Cells(40)),
        height: Dimension::Length(Length::Cells(10)),
        ..Default::default()
    };
    doc.set_style(child1, child_style.clone());
    doc.set_style(child2, child_style.clone());
    doc.set_style(child3, child_style);

    compute_layout(&mut doc, root, Size::new(100, 100));

    // First line at 0, second line at end
    assert_eq!(doc.layout(child1).location.y, 0);
    assert_eq!(doc.layout(child3).location.y, 90);
}

#[test]
fn flex_align_content_stretch() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    let child3 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_content: AlignContent::Stretch,
            height: Dimension::Length(Length::Cells(100)),
            ..Default::default()
        },
    );
    let child_style = ComputedStyle {
        width: Dimension::Length(Length::Cells(40)),
        height: Dimension::Length(Length::Cells(10)),
        ..Default::default()
    };
    doc.set_style(child1, child_style.clone());
    doc.set_style(child2, child_style.clone());
    doc.set_style(child3, child_style);

    compute_layout(&mut doc, root, Size::new(100, 100));

    // Lines stretched - each line gets extra space
    // 2 lines, 100 height, each line gets 50
    assert_eq!(doc.layout(child1).location.y, 0);
    assert_eq!(doc.layout(child3).location.y, 50);
}

// =============================================================================
// FLEX GAP TESTS
// =============================================================================

#[test]
fn flex_column_gap() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: Length::Cells(10),
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location.x, 0);
    assert_eq!(doc.layout(child2).location.x, 30);
}

#[test]
fn flex_row_gap() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    let child3 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: Length::Cells(5),
            ..Default::default()
        },
    );
    let child_style = ComputedStyle {
        width: Dimension::Length(Length::Cells(40)),
        height: Dimension::Length(Length::Cells(10)),
        ..Default::default()
    };
    doc.set_style(child1, child_style.clone());
    doc.set_style(child2, child_style.clone());
    doc.set_style(child3, child_style);

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location.y, 0);
    assert_eq!(doc.layout(child3).location.y, 15); // 10 + 5 gap
}

// =============================================================================
// BOX MODEL TESTS
// =============================================================================

#[test]
fn padding_affects_content_size() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            display: Display::Block,
            width: Dimension::Length(Length::Cells(50)),
            height: Dimension::Length(Length::Cells(30)),
            padding: Edges::all(Length::Cells(5)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    let layout = doc.layout(child);
    assert_eq!(layout.resolved_box.padding, Edges::all(5));
    assert_eq!(layout.resolved_box.border_box_size().width, 60); // 50 + 5 + 5
    assert_eq!(layout.resolved_box.border_box_size().height, 40); // 30 + 5 + 5
}

#[test]
fn border_affects_content_size() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            display: Display::Block,
            width: Dimension::Length(Length::Cells(50)),
            height: Dimension::Length(Length::Cells(30)),
            border_style: Edges::all(BorderStyle::Solid),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    let layout = doc.layout(child);
    assert_eq!(layout.resolved_box.border, Edges::all(1));
    assert_eq!(layout.resolved_box.border_box_size().width, 52); // 50 + 1 + 1
    assert_eq!(layout.resolved_box.border_box_size().height, 32); // 30 + 1 + 1
}

#[test]
fn nested_containers() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let outer = doc.create_element("div");
    let inner = doc.create_element("div");
    doc.append_child(root, outer);
    doc.append_child(outer, inner);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );
    doc.set_style(
        outer,
        ComputedStyle {
            display: Display::Block,
            padding: Edges::all(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        inner,
        ComputedStyle {
            display: Display::Block,
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(20)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    // Inner is positioned relative to outer's content box
    assert_eq!(doc.layout(inner).location, Point::new(0, 0));
    // Outer's content box starts after padding
    assert_eq!(doc.layout(outer).resolved_box.padding, Edges::all(10));
}

// =============================================================================
// OTHER TESTS
// =============================================================================

#[test]
fn display_none_skips_node() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("div");
    let child2 = doc.create_element("div");
    let child3 = doc.create_element("div");
    doc.append_child(root, child1);
    doc.append_child(root, child2);
    doc.append_child(root, child3);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );
    doc.set_style(
        child1,
        ComputedStyle {
            display: Display::Block,
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child2,
        ComputedStyle {
            display: Display::None,
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );
    doc.set_style(
        child3,
        ComputedStyle {
            display: Display::Block,
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child1).location, Point::new(0, 0));
    assert_eq!(doc.layout(child2), Layout::ZERO);
    // child3 should be right after child1, not after hidden child2
    assert_eq!(doc.layout(child3).location, Point::new(0, 10));
}

#[test]
fn percentage_width() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            display: Display::Block,
            width: Dimension::Length(Length::Percent(50.0)),
            height: Dimension::Length(Length::Cells(10)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child).resolved_box.border_box_size().width, 50);
}

#[test]
fn percentage_height() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            height: Dimension::Length(Length::Cells(80)),
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            display: Display::Block,
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Percent(50.0)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 80));

    assert_eq!(doc.layout(child).resolved_box.border_box_size().height, 40);
}

#[test]
fn min_height_constraint() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            display: Display::Block,
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(10)),
            min_height: Dimension::Length(Length::Cells(30)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child).resolved_box.border_box_size().height, 30);
}

#[test]
fn max_height_constraint() {
    let mut doc = MockDocument::new();
    let root = doc.create_element("div");
    let child = doc.create_element("div");
    doc.append_child(root, child);
    doc.set_style(
        root,
        ComputedStyle {
            display: Display::Block,
            ..Default::default()
        },
    );
    doc.set_style(
        child,
        ComputedStyle {
            display: Display::Block,
            width: Dimension::Length(Length::Cells(20)),
            height: Dimension::Length(Length::Cells(50)),
            max_height: Dimension::Length(Length::Cells(30)),
            ..Default::default()
        },
    );

    compute_layout(&mut doc, root, Size::new(100, 100));

    assert_eq!(doc.layout(child).resolved_box.border_box_size().height, 30);
}
