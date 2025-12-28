//! End-to-end integration tests for the Bulma style system.

use std::collections::HashMap;

use capsule_corp::{
    Bulma, Color, ComputedStyle, CustomPropertiesMap, Display, ElementState, FlexDirection,
    FontWeight, Length, Overflow, RestyleHint, Stylesheet, TDocument, TElement, TextAlign,
    Visibility, compute_styles, restyle_subtree,
};
use ginyu_force::Pose;

#[derive(Debug, Clone)]
struct MockElement {
    node_id: usize,
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

#[derive(Debug, Clone)]
struct MockElementData {
    tag: Pose,
    id: Option<Pose>,
    classes: Vec<Pose>,
    state: ElementState,
}

impl MockElement {
    fn new(node_id: usize, tag: &str) -> Self {
        Self {
            node_id,
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
        }
    }
}

impl PartialEq for MockElement {
    fn eq(&self, other: &Self) -> bool {
        self.node_id == other.node_id
    }
}

impl TElement for MockElement {
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
        self.parent_data.as_ref().map(|data| {
            let mut el = Self::new(usize::MAX, data.tag.as_str());
            el.id = data.id;
            el.classes.clone_from(&data.classes);
            el.state = data.state;
            el
        })
    }

    fn prev_sibling(&self) -> Option<Self> {
        self.prev_sibling_data.as_ref().map(|data| {
            let mut el = Self::new(usize::MAX, data.tag.as_str());
            el.id = data.id;
            el.classes.clone_from(&data.classes);
            el.state = data.state;
            el
        })
    }

    fn next_sibling(&self) -> Option<Self> {
        self.next_sibling_data.as_ref().map(|data| {
            let mut el = Self::new(usize::MAX, data.tag.as_str());
            el.id = data.id;
            el.classes = data.classes.clone();
            el.state = data.state;
            el
        })
    }

    fn has_children(&self) -> bool {
        self.has_children
    }
}

struct MockDocument {
    nodes: HashMap<usize, MockElement>,
    children: HashMap<usize, Vec<usize>>,
    parent: HashMap<usize, usize>,
    styles: HashMap<usize, (ComputedStyle, CustomPropertiesMap)>,
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
            styles: HashMap::new(),
            stylist: Bulma::new(),
            root: 0,
            next_id: 0,
        }
    }

    fn with_stylesheets(css: &[&str]) -> Self {
        let mut doc = Self::new();
        for source in css {
            let stylesheet = Stylesheet::parse(source).expect("invalid stylesheet");
            doc.stylist.add_stylesheet(&stylesheet);
        }
        doc
    }

    fn with_ua_and_author(ua: &str, author: &str) -> Self {
        let mut doc = Self::new();
        let ua_sheet = Stylesheet::parse(ua).expect("invalid UA stylesheet");
        let author_sheet = Stylesheet::parse(author).expect("invalid author stylesheet");
        doc.stylist.add_ua_stylesheet(&ua_sheet);
        doc.stylist.add_stylesheet(&author_sheet);
        doc
    }

    fn create_element(&mut self, tag: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, MockElement::new(id, tag));
        if id == 0 {
            self.root = id;
        }
        id
    }

    fn append_child(&mut self, parent: usize, child: usize) {
        if let Some(parent_el) = self.nodes.get_mut(&parent) {
            parent_el.has_children = true;
        }

        if let Some(parent_el) = self.nodes.get(&parent) {
            let parent_data = MockElementData {
                tag: parent_el.tag,
                id: parent_el.id,
                classes: parent_el.classes.clone(),
                state: parent_el.state,
            };
            if let Some(child_el) = self.nodes.get_mut(&child) {
                child_el.parent_data = Some(Box::new(parent_data));
            }
        }

        let siblings = self.children.entry(parent).or_default();
        if let Some(&prev_id) = siblings.last() {
            if let Some(prev_el) = self.nodes.get(&prev_id) {
                let prev_data = MockElementData {
                    tag: prev_el.tag,
                    id: prev_el.id,
                    classes: prev_el.classes.clone(),
                    state: prev_el.state,
                };
                if let Some(child_el) = self.nodes.get_mut(&child) {
                    child_el.prev_sibling_data = Some(Box::new(prev_data));
                }
            }

            if let Some(child_el) = self.nodes.get(&child) {
                let next_data = MockElementData {
                    tag: child_el.tag,
                    id: child_el.id,
                    classes: child_el.classes.clone(),
                    state: child_el.state,
                };
                if let Some(prev_el) = self.nodes.get_mut(&prev_id) {
                    prev_el.next_sibling_data = Some(Box::new(next_data));
                }
            }
        }

        siblings.push(child);
        self.parent.insert(child, parent);
    }

    fn set_id(&mut self, node: usize, id: &str) {
        if let Some(el) = self.nodes.get_mut(&node) {
            el.id = Some(Pose::from(id));
        }
    }

    fn add_class(&mut self, node: usize, class: &str) {
        if let Some(el) = self.nodes.get_mut(&node) {
            el.classes.push(Pose::from(class));
        }
    }

    fn set_inline_style(&mut self, node: usize, style: &str) {
        if let Some(el) = self.nodes.get_mut(&node) {
            el.inline_style = Some(style.to_string());
        }
    }

    fn set_state(&mut self, node: usize, state: ElementState) {
        if let Some(el) = self.nodes.get_mut(&node) {
            el.state = state;
        }
    }

    fn style(&self, node: usize) -> Option<&ComputedStyle> {
        self.styles.get(&node).map(|(s, _)| s)
    }

    fn custom_props(&self, node: usize) -> Option<&CustomPropertiesMap> {
        self.styles.get(&node).map(|(_, cp)| cp)
    }
}

impl TDocument for MockDocument {
    type Element = MockElement;
    type NodeId = usize;

    fn root(&self) -> usize {
        self.root
    }

    fn as_element(&self, node: usize) -> MockElement {
        self.nodes.get(&node).cloned().expect("node not found")
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
        self.styles.get(&node).map(|(s, _)| s)
    }

    fn custom_properties(&self, node: usize) -> Option<&CustomPropertiesMap> {
        self.styles.get(&node).map(|(_, cp)| cp)
    }

    fn set_style(&mut self, node: usize, style: ComputedStyle, cp: CustomPropertiesMap) {
        self.styles.insert(node, (style, cp));
    }

    fn take_stylist(&mut self) -> Bulma {
        std::mem::take(&mut self.stylist)
    }

    fn set_stylist(&mut self, stylist: Bulma) {
        self.stylist = stylist;
    }
}

const RESET_STYLESHEET: &str = r"
* {
    margin: 0;
    padding: 0;
}

:root {
    --color-primary: blue;
    --color-secondary: cyan;
    --color-danger: red;
    --color-success: green;
    --color-warning: yellow;
    --color-text: white;
    --color-bg: black;
    --spacing-sm: 1;
    --spacing-md: 2;
    --spacing-lg: 4;
}
";

const COMPONENT_STYLESHEET: &str = r"
.container {
    display: flex;
    flex-direction: column;
    padding: var(--spacing-md);
}

.row {
    display: flex;
    flex-direction: row;
}

.col {
    display: flex;
    flex-direction: column;
}

.btn {
    display: flex;
    padding: var(--spacing-sm);
    color: var(--color-text);
    background: var(--color-primary);

    &:hover {
        background: var(--color-secondary);
    }

    &:disabled {
        background: reset;
        color: reset;
    }

    &.btn-danger {
        background: var(--color-danger);
    }

    &.btn-success {
        background: var(--color-success);
    }
}

.card {
    display: flex;
    flex-direction: column;
    border: solid var(--color-text);
    padding: var(--spacing-md);

    .card-header {
        font-weight: bold;
        padding: var(--spacing-sm);
    }

    .card-body {
        padding: var(--spacing-md);
    }

    .card-footer {
        padding: var(--spacing-sm);
    }
}

.text-center { text-align: center }
.text-left { text-align: left }
.text-right { text-align: right }

.font-bold { font-weight: bold }

.hidden { visibility: hidden }
.flex { display: flex }
.block { display: block }
.none { display: none }

.overflow-hidden { overflow: hidden }
.overflow-scroll { overflow: scroll }

.gap-sm { gap: var(--spacing-sm) }
.gap-md { gap: var(--spacing-md) }
.gap-lg { gap: var(--spacing-lg) }

.p-sm { padding: var(--spacing-sm) }
.p-md { padding: var(--spacing-md) }
.p-lg { padding: var(--spacing-lg) }

.m-sm { margin: var(--spacing-sm) }
.m-md { margin: var(--spacing-md) }
.m-lg { margin: var(--spacing-lg) }

#main-content {
    display: flex;
    flex-direction: column;
    width: 100%;
    height: 100%;
}

#sidebar {
    display: flex;
    flex-direction: column;
    width: 20;
}

#header {
    display: flex;
    height: 3;
    background: var(--color-primary);
}

#footer {
    display: flex;
    height: 1;
}
";

const fn framework_stylesheets() -> [&'static str; 2] {
    [RESET_STYLESHEET, COMPONENT_STYLESHEET]
}

mod basic_styling {
    use super::*;

    #[test]
    fn style_single_element() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        doc.add_class(root, "container");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.display, Display::Flex);
        assert_eq!(style.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn style_parent_child() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let parent = doc.create_element("div");
        let child = doc.create_element("span");
        doc.add_class(parent, "container");
        doc.append_child(parent, child);

        compute_styles(&mut doc);

        let parent_style = doc.style(parent).expect("no parent style");
        assert_eq!(parent_style.display, Display::Flex);

        let child_style = doc.style(child).expect("no child style");
        assert_eq!(child_style.display, Display::default());
    }

    #[test]
    fn style_deep_tree() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let card = doc.create_element("div");
        let header = doc.create_element("div");
        let body = doc.create_element("div");
        let footer = doc.create_element("div");

        doc.add_class(root, "container");
        doc.add_class(card, "card");
        doc.add_class(header, "card-header");
        doc.add_class(body, "card-body");
        doc.add_class(footer, "card-footer");

        doc.append_child(root, card);
        doc.append_child(card, header);
        doc.append_child(card, body);
        doc.append_child(card, footer);

        compute_styles(&mut doc);

        for id in [root, card, header, body, footer] {
            assert!(doc.style(id).is_some(), "node {id} missing style");
        }

        let header_style = doc.style(header).expect("no header style");
        assert_eq!(header_style.font_weight, FontWeight::Bold);
    }

    #[test]
    fn style_with_ids() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let main = doc.create_element("main");
        let sidebar = doc.create_element("aside");

        doc.set_id(main, "main-content");
        doc.set_id(sidebar, "sidebar");

        doc.append_child(root, main);
        doc.append_child(root, sidebar);

        compute_styles(&mut doc);

        let main_style = doc.style(main).expect("no main style");
        assert_eq!(main_style.display, Display::Flex);
        assert_eq!(main_style.flex_direction, FlexDirection::Column);

        let sidebar_style = doc.style(sidebar).expect("no sidebar style");
        assert_eq!(
            sidebar_style.width,
            capsule_corp::Dimension::Length(Length::Cells(20))
        );
    }
}

mod custom_properties {
    use super::*;

    #[test]
    fn root_custom_properties_available() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");

        compute_styles(&mut doc);

        let cp = doc.custom_props(root).expect("no custom props");
        assert_eq!(cp.get(Pose::from("color-primary")), Some("blue"));
        assert_eq!(cp.get(Pose::from("spacing-md")), Some("2"));
    }

    #[test]
    fn custom_properties_resolve_in_values() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let btn = doc.create_element("button");

        doc.add_class(btn, "btn");
        doc.append_child(root, btn);

        compute_styles(&mut doc);

        let btn_style = doc.style(btn).expect("no btn style");
        assert_eq!(btn_style.background_color, Color::BLUE);
    }

    #[test]
    fn custom_properties_inherit() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let child = doc.create_element("div");
        let grandchild = doc.create_element("div");

        doc.append_child(root, child);
        doc.append_child(child, grandchild);

        compute_styles(&mut doc);

        let gcp = doc.custom_props(grandchild).expect("no custom props");
        assert_eq!(gcp.get(Pose::from("color-primary")), Some("blue"));
    }

    #[test]
    fn custom_properties_can_be_overridden() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let child = doc.create_element("div");

        doc.set_inline_style(child, "--color-primary: magenta");
        doc.append_child(root, child);

        compute_styles(&mut doc);

        let child_cp = doc.custom_props(child).expect("no custom props");
        assert_eq!(child_cp.get(Pose::from("color-primary")), Some("magenta"));
    }

    #[test]
    fn var_with_fallback() {
        let mut doc = MockDocument::with_stylesheets(&[""]);
        let root = doc.create_element("div");

        doc.set_inline_style(root, "color: var(--nonexistent, cyan)");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.color, Color::CYAN);
    }
}

mod cascade {
    use super::*;

    #[test]
    fn class_beats_tag() {
        let mut doc =
            MockDocument::with_stylesheets(&["div { display: block }", ".btn { display: flex }"]);
        let root = doc.create_element("div");
        doc.add_class(root, "btn");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.display, Display::Flex);
    }

    #[test]
    fn id_beats_class() {
        let mut doc = MockDocument::with_stylesheets(&[
            ".btn { display: flex }",
            "#special { display: block }",
        ]);
        let root = doc.create_element("div");
        doc.add_class(root, "btn");
        doc.set_id(root, "special");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.display, Display::Block);
    }

    #[test]
    fn important_beats_specificity() {
        let mut doc = MockDocument::with_stylesheets(&[
            ".btn { display: flex !important }",
            "#special { display: block }",
        ]);
        let root = doc.create_element("div");
        doc.add_class(root, "btn");
        doc.set_id(root, "special");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.display, Display::Flex);
    }

    #[test]
    fn inline_beats_stylesheet() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        doc.add_class(root, "btn");
        doc.set_inline_style(root, "display: block");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.display, Display::Block);
    }

    #[test]
    fn stylesheet_important_beats_inline() {
        let mut doc = MockDocument::with_stylesheets(&[".override { display: flex !important }"]);
        let root = doc.create_element("div");
        doc.add_class(root, "override");
        doc.set_inline_style(root, "display: block");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.display, Display::Flex);
    }

    #[test]
    fn inline_important_beats_all() {
        let mut doc = MockDocument::with_stylesheets(&[".override { display: flex !important }"]);
        let root = doc.create_element("div");
        doc.add_class(root, "override");
        doc.set_inline_style(root, "display: block !important");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.display, Display::Block);
    }

    #[test]
    fn author_beats_ua() {
        let mut doc =
            MockDocument::with_ua_and_author("div { display: block }", "div { display: flex }");
        let root = doc.create_element("div");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.display, Display::Flex);
    }
}

mod pseudo_classes {
    use super::*;

    #[test]
    fn hover_state() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let btn = doc.create_element("button");

        doc.add_class(btn, "btn");
        doc.set_state(btn, ElementState::HOVER);
        doc.append_child(root, btn);

        compute_styles(&mut doc);

        let btn_style = doc.style(btn).expect("no btn style");
        assert_eq!(btn_style.background_color, Color::CYAN);
    }

    #[test]
    fn disabled_state() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let btn = doc.create_element("button");

        doc.add_class(btn, "btn");
        doc.set_state(btn, ElementState::DISABLED);
        doc.append_child(root, btn);

        compute_styles(&mut doc);

        let btn_style = doc.style(btn).expect("no btn style");
        assert_eq!(btn_style.background_color, Color::Reset);
        assert_eq!(btn_style.color, Color::Reset);
    }

    #[test]
    fn combined_class_and_pseudo() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let btn = doc.create_element("button");

        doc.add_class(btn, "btn");
        doc.add_class(btn, "btn-danger");
        doc.append_child(root, btn);

        compute_styles(&mut doc);

        let btn_style = doc.style(btn).expect("no btn style");
        assert_eq!(btn_style.background_color, Color::RED);
    }

    #[test]
    fn first_child() {
        let mut doc = MockDocument::with_stylesheets(&["li:first-child { color: red }"]);
        let root = doc.create_element("ul");
        let li1 = doc.create_element("li");
        let li2 = doc.create_element("li");
        let li3 = doc.create_element("li");

        doc.append_child(root, li1);
        doc.append_child(root, li2);
        doc.append_child(root, li3);

        compute_styles(&mut doc);

        assert_eq!(doc.style(li1).expect("no style").color, Color::RED);
        assert_eq!(doc.style(li2).expect("no style").color, Color::Reset);
        assert_eq!(doc.style(li3).expect("no style").color, Color::Reset);
    }

    #[test]
    fn last_child() {
        let mut doc = MockDocument::with_stylesheets(&["li:last-child { color: blue }"]);
        let root = doc.create_element("ul");
        let li1 = doc.create_element("li");
        let li2 = doc.create_element("li");
        let li3 = doc.create_element("li");

        doc.append_child(root, li1);
        doc.append_child(root, li2);
        doc.append_child(root, li3);

        compute_styles(&mut doc);

        assert_eq!(doc.style(li1).expect("no style").color, Color::Reset);
        assert_eq!(doc.style(li2).expect("no style").color, Color::Reset);
        assert_eq!(doc.style(li3).expect("no style").color, Color::BLUE);
    }
}

mod inheritance {
    use super::*;

    #[test]
    fn color_inherits() {
        let mut doc = MockDocument::with_stylesheets(&[""]);
        let root = doc.create_element("div");
        let child = doc.create_element("span");

        doc.set_inline_style(root, "color: magenta");
        doc.append_child(root, child);

        compute_styles(&mut doc);

        let child_style = doc.style(child).expect("no style");
        assert_eq!(child_style.color, Color::MAGENTA);
    }

    #[test]
    fn font_weight_inherits() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let child = doc.create_element("span");

        doc.add_class(root, "font-bold");
        doc.append_child(root, child);

        compute_styles(&mut doc);

        let child_style = doc.style(child).expect("no style");
        assert_eq!(child_style.font_weight, FontWeight::Bold);
    }

    #[test]
    fn text_align_inherits() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let child = doc.create_element("p");

        doc.add_class(root, "text-center");
        doc.append_child(root, child);

        compute_styles(&mut doc);

        let child_style = doc.style(child).expect("no style");
        assert_eq!(child_style.text_align, TextAlign::Center);
    }

    #[test]
    fn display_does_not_inherit() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let child = doc.create_element("div");

        doc.add_class(root, "flex");
        doc.append_child(root, child);

        compute_styles(&mut doc);

        let child_style = doc.style(child).expect("no style");
        assert_eq!(child_style.display, Display::default());
    }

    #[test]
    fn inherit_keyword_forces_inheritance() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let child = doc.create_element("div");

        doc.add_class(root, "flex");
        doc.set_inline_style(child, "display: inherit");
        doc.append_child(root, child);

        compute_styles(&mut doc);

        let child_style = doc.style(child).expect("no style");
        assert_eq!(child_style.display, Display::Flex);
    }

    #[test]
    fn initial_keyword_resets() {
        let mut doc = MockDocument::with_stylesheets(&[""]);
        let root = doc.create_element("div");
        let child = doc.create_element("span");

        doc.set_inline_style(root, "color: red");
        doc.set_inline_style(child, "color: initial");
        doc.append_child(root, child);

        compute_styles(&mut doc);

        let child_style = doc.style(child).expect("no style");
        assert_eq!(child_style.color, Color::Reset);
    }

    #[test]
    fn unset_on_inherited_property_inherits() {
        let mut doc = MockDocument::with_stylesheets(&[""]);
        let root = doc.create_element("div");
        let child = doc.create_element("span");

        doc.set_inline_style(root, "color: cyan");
        doc.set_inline_style(child, "color: unset");
        doc.append_child(root, child);

        compute_styles(&mut doc);

        let child_style = doc.style(child).expect("no style");
        // unset on inherited property = inherit
        assert_eq!(child_style.color, Color::CYAN);
    }

    #[test]
    fn unset_on_non_inherited_property_resets() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let child = doc.create_element("div");

        doc.add_class(root, "flex");
        doc.set_inline_style(child, "display: unset");
        doc.append_child(root, child);

        compute_styles(&mut doc);

        let child_style = doc.style(child).expect("no style");
        // unset on non-inherited property = initial
        assert_eq!(child_style.display, Display::default());
    }

    #[test]
    fn unset_overrides_previous_value() {
        let mut doc = MockDocument::with_stylesheets(&["div { color: red }"]);
        let root = doc.create_element("div");
        let child = doc.create_element("div");

        doc.set_inline_style(root, "color: blue");
        doc.set_inline_style(child, "color: unset");
        doc.append_child(root, child);

        compute_styles(&mut doc);

        let child_style = doc.style(child).expect("no style");
        // unset should inherit from parent, not use the stylesheet value
        assert_eq!(child_style.color, Color::BLUE);
    }
}

mod restyle {
    use super::*;

    #[test]
    fn restyle_single_node() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        doc.add_class(root, "btn");

        compute_styles(&mut doc);
        assert_eq!(
            doc.style(root).expect("no style").background_color,
            Color::BLUE
        );

        doc.set_state(root, ElementState::HOVER);
        restyle_subtree(&mut doc, root, RestyleHint::RESTYLE_SELF);

        assert_eq!(
            doc.style(root).expect("no style").background_color,
            Color::CYAN
        );
    }

    #[test]
    fn restyle_descendants() {
        let mut doc = MockDocument::with_stylesheets(&["div { color: red }"]);
        let root = doc.create_element("div");
        let child = doc.create_element("div");
        let grandchild = doc.create_element("div");

        doc.append_child(root, child);
        doc.append_child(child, grandchild);

        compute_styles(&mut doc);

        doc.stylist.clear();
        let new_sheet = Stylesheet::parse("div { color: blue }").expect("parse");
        doc.stylist.add_stylesheet(&new_sheet);

        restyle_subtree(
            &mut doc,
            root,
            RestyleHint::RESTYLE_SELF | RestyleHint::RESTYLE_DESCENDANTS,
        );

        assert_eq!(doc.style(root).expect("no style").color, Color::BLUE);
        assert_eq!(doc.style(child).expect("no style").color, Color::BLUE);
        assert_eq!(doc.style(grandchild).expect("no style").color, Color::BLUE);
    }

    #[test]
    fn restyle_empty_hint_noop() {
        let mut doc = MockDocument::with_stylesheets(&[""]);
        let root = doc.create_element("div");

        restyle_subtree(&mut doc, root, RestyleHint::empty());

        assert!(doc.style(root).is_none());
    }

    #[test]
    fn restyle_after_class_change() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let btn = doc.create_element("button");

        doc.add_class(btn, "btn");
        doc.append_child(root, btn);

        compute_styles(&mut doc);
        assert_eq!(
            doc.style(btn).expect("no style").background_color,
            Color::BLUE
        );

        doc.add_class(btn, "btn-danger");
        restyle_subtree(&mut doc, btn, RestyleHint::RESTYLE_SELF);

        assert_eq!(
            doc.style(btn).expect("no style").background_color,
            Color::RED
        );
    }
}

mod complex_layouts {
    use super::*;

    #[test]
    fn app_layout() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());

        let root = doc.create_element("div");
        let header = doc.create_element("header");
        let main_area = doc.create_element("div");
        let sidebar = doc.create_element("aside");
        let content = doc.create_element("main");
        let footer = doc.create_element("footer");

        doc.set_id(header, "header");
        doc.set_id(sidebar, "sidebar");
        doc.set_id(content, "main-content");
        doc.set_id(footer, "footer");
        doc.add_class(main_area, "row");

        doc.append_child(root, header);
        doc.append_child(root, main_area);
        doc.append_child(main_area, sidebar);
        doc.append_child(main_area, content);
        doc.append_child(root, footer);

        compute_styles(&mut doc);

        let header_style = doc.style(header).expect("no header style");
        assert_eq!(header_style.display, Display::Flex);
        assert_eq!(
            header_style.height,
            capsule_corp::Dimension::Length(Length::Cells(3))
        );

        let main_area_style = doc.style(main_area).expect("no main area style");
        assert_eq!(main_area_style.display, Display::Flex);
        assert_eq!(main_area_style.flex_direction, FlexDirection::Row);

        let sidebar_style = doc.style(sidebar).expect("no sidebar style");
        assert_eq!(
            sidebar_style.width,
            capsule_corp::Dimension::Length(Length::Cells(20))
        );

        let content_style = doc.style(content).expect("no content style");
        assert_eq!(content_style.display, Display::Flex);
        assert_eq!(content_style.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn nested_cards() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());

        let root = doc.create_element("div");
        doc.add_class(root, "container");

        for _ in 0..3 {
            let card = doc.create_element("div");
            let card_header = doc.create_element("div");
            let card_body = doc.create_element("div");

            doc.add_class(card, "card");
            doc.add_class(card_header, "card-header");
            doc.add_class(card_body, "card-body");

            doc.append_child(root, card);
            doc.append_child(card, card_header);
            doc.append_child(card, card_body);
        }

        compute_styles(&mut doc);

        for id in 0..doc.next_id {
            assert!(doc.style(id).is_some(), "node {id} missing style");
        }

        let card_header_id = 2;
        let header_style = doc.style(card_header_id).expect("no style");
        assert_eq!(header_style.font_weight, FontWeight::Bold);
    }

    #[test]
    fn utility_classes_combine() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");

        doc.add_class(root, "flex");
        doc.add_class(root, "text-center");
        doc.add_class(root, "font-bold");
        doc.add_class(root, "p-lg");
        doc.add_class(root, "overflow-hidden");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.display, Display::Flex);
        assert_eq!(style.text_align, TextAlign::Center);
        assert_eq!(style.font_weight, FontWeight::Bold);
        assert_eq!(style.padding.top, Length::Cells(4));
        assert_eq!(style.overflow_x, Overflow::Hidden);
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn empty_document() {
        let mut doc = MockDocument::with_stylesheets(&[""]);
        let root = doc.create_element("div");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.display, Display::default());
    }

    #[test]
    fn deeply_nested_tree() {
        let mut doc = MockDocument::with_stylesheets(&[""]);
        let mut parent = doc.create_element("div");
        doc.set_inline_style(parent, "color: cyan");

        for _ in 0..20 {
            let child = doc.create_element("div");
            doc.append_child(parent, child);
            parent = child;
        }

        compute_styles(&mut doc);

        let deepest_style = doc.style(parent).expect("no style");
        assert_eq!(deepest_style.color, Color::CYAN);
    }

    #[test]
    fn many_siblings() {
        let mut doc = MockDocument::with_stylesheets(&[".even { color: blue }"]);
        let root = doc.create_element("ul");

        for i in 0..100 {
            let li = doc.create_element("li");
            if i % 2 == 0 {
                doc.add_class(li, "even");
            }
            doc.append_child(root, li);
        }

        compute_styles(&mut doc);

        for i in 1..=100 {
            let style = doc.style(i).expect("no style");
            if i % 2 == 1 {
                assert_eq!(style.color, Color::BLUE);
            } else {
                assert_eq!(style.color, Color::Reset);
            }
        }
    }

    #[test]
    fn invalid_var_fallback_chain() {
        let mut doc = MockDocument::with_stylesheets(&[""]);
        let root = doc.create_element("div");

        doc.set_inline_style(root, "color: var(--a, var(--b, var(--c, red)))");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn circular_custom_property_reference() {
        let mut doc = MockDocument::with_stylesheets(&[""]);
        let root = doc.create_element("div");

        doc.set_inline_style(root, "--a: var(--b); --b: var(--a); color: var(--a, blue)");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.color, Color::BLUE);
    }

    #[test]
    fn multiple_classes_same_property() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");

        doc.add_class(root, "text-left");
        doc.add_class(root, "text-center");
        doc.add_class(root, "text-right");

        compute_styles(&mut doc);

        let style = doc.style(root).expect("no style");
        assert_eq!(style.text_align, TextAlign::Right);
    }

    #[test]
    fn visibility_hidden_still_computes() {
        let mut doc = MockDocument::with_stylesheets(&framework_stylesheets());
        let root = doc.create_element("div");
        let hidden = doc.create_element("div");
        let child_of_hidden = doc.create_element("span");

        doc.add_class(hidden, "hidden");
        doc.set_inline_style(hidden, "color: red");
        doc.append_child(root, hidden);
        doc.append_child(hidden, child_of_hidden);

        compute_styles(&mut doc);

        let hidden_style = doc.style(hidden).expect("no style");
        assert_eq!(hidden_style.visibility, Visibility::Hidden);
        assert_eq!(hidden_style.color, Color::RED);

        let child_style = doc.style(child_of_hidden).expect("no style");
        assert_eq!(child_style.visibility, Visibility::Hidden);
        assert_eq!(child_style.color, Color::RED);
    }
}
