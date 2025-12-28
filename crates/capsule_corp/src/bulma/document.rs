use selectors::context::SelectorCaches;

use crate::{Bulma, ComputedStyle, CustomPropertiesMap, TElement, bulma::restyle::RestyleHint};

pub trait TDocument {
    type Element: TElement;
    type NodeId: Copy + Eq;

    fn root(&self) -> Self::NodeId;
    fn as_element(&self, node: Self::NodeId) -> Self::Element;
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

pub fn compute_styles<D: TDocument>(document: &mut D) {
    let mut stylist = document.take_stylist();
    let mut caches = SelectorCaches::default();
    let root = document.root();

    compute_styles_recursive(document, &mut stylist, &mut caches, root, None, None);

    document.set_stylist(stylist);
}

pub fn restyle_subtree<D: TDocument>(document: &mut D, node: D::NodeId, hint: RestyleHint) {
    if hint.is_empty() {
        return;
    }

    let mut stylist = document.take_stylist();
    let mut caches = SelectorCaches::default();

    restyle_subtree_inner(document, &mut stylist, &mut caches, node, hint);

    document.set_stylist(stylist);
}

fn restyle_subtree_inner<D: TDocument>(
    document: &mut D,
    stylist: &mut Bulma,
    caches: &mut SelectorCaches,
    node: D::NodeId,
    hint: RestyleHint,
) {
    let (parent_style, parent_custom_properties) =
        document.parent(node).map_or((None, None), |parent_id| {
            (
                document.computed_style(parent_id),
                document.custom_properties(parent_id),
            )
        });

    if hint.affects_self() {
        let element = document.as_element(node);
        let (style, custom_properties) =
            stylist.compute_style(&element, parent_style, parent_custom_properties, caches);

        document.set_style(node, style, custom_properties);
    }

    if hint.affects_descendants() {
        let style = document.computed_style(node).cloned();
        let custom_properties = document.custom_properties(node).cloned();

        let children: Vec<_> = document.children(node).collect();
        for child in children {
            restyle_subtree_recursive(
                document,
                stylist,
                caches,
                child,
                style.as_ref(),
                custom_properties.as_ref(),
            );
        }
    }

    if hint.affects_later_siblings() {
        let siblings: Vec<_> = document.next_siblings(node).collect();
        for sibling in siblings {
            let sibling_hint = hint.propagate_to_later_sibling();
            restyle_subtree_inner(document, stylist, caches, sibling, sibling_hint);
        }
    }
}

fn compute_styles_recursive<D: TDocument>(
    document: &mut D,
    stylist: &mut Bulma,
    caches: &mut SelectorCaches,
    node: D::NodeId,
    parent_style: Option<&ComputedStyle>,
    parent_custom_properties: Option<&CustomPropertiesMap>,
) {
    let element = document.as_element(node);
    let (style, custom_properties) =
        stylist.compute_style(&element, parent_style, parent_custom_properties, caches);

    let children: Vec<_> = document.children(node).collect();

    document.set_style(node, style.clone(), custom_properties.clone());

    for child in children {
        compute_styles_recursive(
            document,
            stylist,
            caches,
            child,
            Some(&style),
            Some(&custom_properties),
        );
    }
}

fn restyle_subtree_recursive<D: TDocument>(
    document: &mut D,
    stylist: &mut Bulma,
    caches: &mut SelectorCaches,
    node: D::NodeId,
    parent_style: Option<&ComputedStyle>,
    parent_custom_properties: Option<&CustomPropertiesMap>,
) {
    let element = document.as_element(node);
    let (style, custom_properties) =
        stylist.compute_style(&element, parent_style, parent_custom_properties, caches);

    let children: Vec<_> = document.children(node).collect();

    document.set_style(node, style.clone(), custom_properties.clone());

    for child in children {
        restyle_subtree_recursive(
            document,
            stylist,
            caches,
            child,
            Some(&style),
            Some(&custom_properties),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Display, ElementState, Stylesheet};
    use ginyu_force::Pose;
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq)]
    struct TestEl {
        id: usize,
        tag: Pose,
        class: Option<Pose>,
        element_id: Option<Pose>,
        state: ElementState,
        parent_id: Option<usize>,
    }

    impl TElement for TestEl {
        fn tag_name(&self) -> Pose {
            self.tag
        }

        fn id(&self) -> Option<Pose> {
            self.element_id
        }

        fn has_class(&self, name: &str) -> bool {
            self.class.is_some_and(|c| c.as_str() == name)
        }

        fn each_class<F: FnMut(Pose)>(&self, mut f: F) {
            if let Some(c) = self.class {
                f(c);
            }
        }

        fn get_attribute(&self, _name: Pose) -> Option<&str> {
            None
        }

        fn style_attribute(&self) -> Option<&str> {
            None
        }

        fn state(&self) -> ElementState {
            self.state
        }

        fn parent(&self) -> Option<Self> {
            None
        }

        fn prev_sibling(&self) -> Option<Self> {
            None
        }

        fn next_sibling(&self) -> Option<Self> {
            None
        }

        fn has_children(&self) -> bool {
            false
        }
    }

    struct TestDoc {
        nodes: HashMap<usize, TestEl>,
        children: HashMap<usize, Vec<usize>>,
        styles: HashMap<usize, (ComputedStyle, CustomPropertiesMap)>,
        stylist: Bulma,
        root: usize,
    }

    impl TestDoc {
        fn new() -> Self {
            Self {
                nodes: HashMap::new(),
                children: HashMap::new(),
                styles: HashMap::new(),
                stylist: Bulma::new(),
                root: 0,
            }
        }

        fn add_node(&mut self, id: usize, tag: &str, parent: Option<usize>) -> &mut Self {
            self.nodes.insert(
                id,
                TestEl {
                    id,
                    tag: Pose::from(tag),
                    class: None,
                    element_id: None,
                    state: ElementState::empty(),
                    parent_id: parent,
                },
            );
            if let Some(p) = parent {
                self.children.entry(p).or_default().push(id);
            }
            self
        }

        fn with_class(mut self, id: usize, class: &str) -> Self {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.class = Some(Pose::from(class));
            }
            self
        }

        fn with_stylesheet(mut self, css: &str) -> Self {
            let stylesheet = Stylesheet::parse(css).expect("parse");
            self.stylist.add_stylesheet(&stylesheet);
            self
        }
    }

    impl TDocument for TestDoc {
        type Element = TestEl;
        type NodeId = usize;

        fn root(&self) -> usize {
            self.root
        }

        fn as_element(&self, node: usize) -> TestEl {
            self.nodes.get(&node).cloned().expect("node not found")
        }

        fn parent(&self, node: usize) -> Option<usize> {
            self.nodes.get(&node).and_then(|n| n.parent_id)
        }

        fn children(&self, node: usize) -> impl Iterator<Item = usize> {
            self.children
                .get(&node)
                .map(|v| v.iter().copied())
                .into_iter()
                .flatten()
        }

        fn next_siblings(&self, node: usize) -> impl Iterator<Item = usize> {
            let parent = self.parent(node);
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

    #[test]
    fn compute_styles_single_node() {
        let mut doc = TestDoc::new();
        doc.add_node(0, "div", None);
        let mut doc = doc.with_stylesheet("div { color: red }");

        compute_styles(&mut doc);

        let style = doc.computed_style(0).expect("no style");
        assert_eq!(style.color, Color::RED);
    }

    #[test]
    fn compute_styles_with_inheritance() {
        let mut doc = TestDoc::new();
        doc.add_node(0, "div", None);
        doc.add_node(1, "span", Some(0));
        let mut doc = doc.with_stylesheet("div { color: cyan }");

        compute_styles(&mut doc);

        let parent_style = doc.computed_style(0).expect("no parent style");
        assert_eq!(parent_style.color, Color::CYAN);

        let child_style = doc.computed_style(1).expect("no child style");
        assert_eq!(child_style.color, Color::CYAN);
    }

    #[test]
    fn compute_styles_child_override() {
        let mut doc = TestDoc::new();
        doc.add_node(0, "div", None);
        doc.add_node(1, "span", Some(0));
        let mut doc = doc
            .with_class(1, "override")
            .with_stylesheet("div { color: red } .override { color: blue }");

        compute_styles(&mut doc);

        let parent_style = doc.computed_style(0).expect("no parent style");
        assert_eq!(parent_style.color, Color::RED);

        let child_style = doc.computed_style(1).expect("no child style");
        assert_eq!(child_style.color, Color::BLUE);
    }

    #[test]
    fn compute_styles_deep_tree() {
        let mut doc = TestDoc::new();
        doc.add_node(0, "div", None);
        doc.add_node(1, "div", Some(0));
        doc.add_node(2, "div", Some(1));
        doc.add_node(3, "span", Some(2));
        let mut doc = doc.with_stylesheet("div { color: cyan }");

        compute_styles(&mut doc);

        for i in 0..4 {
            assert!(doc.computed_style(i).is_some(), "node {i} missing style");
        }

        let leaf_style = doc.computed_style(3).expect("no leaf style");
        assert_eq!(leaf_style.color, Color::CYAN);
    }

    #[test]
    fn compute_styles_multiple_children() {
        let mut doc = TestDoc::new();
        doc.add_node(0, "div", None);
        doc.add_node(1, "span", Some(0));
        doc.add_node(2, "span", Some(0));
        doc.add_node(3, "span", Some(0));
        let mut doc = doc.with_stylesheet("div { color: magenta }");

        compute_styles(&mut doc);

        for i in 1..4 {
            let style = doc.computed_style(i).expect("no style");
            assert_eq!(style.color, Color::MAGENTA);
        }
    }

    #[test]
    fn compute_styles_custom_properties() {
        let mut doc = TestDoc::new();
        doc.add_node(0, "div", None);
        doc.add_node(1, "span", Some(0));
        let mut doc = doc
            .with_class(0, "root")
            .with_class(1, "child")
            .with_stylesheet(
                r"
            .root { --primary: blue }
            .child { color: var(--primary) }
        ",
            );

        compute_styles(&mut doc);

        let child_style = doc.computed_style(1).expect("no child style");
        assert_eq!(child_style.color, Color::BLUE);
    }

    #[test]
    fn restyle_subtree_self_only() {
        let mut doc = TestDoc::new();
        doc.add_node(0, "div", None);
        doc.add_node(1, "span", Some(0));
        let mut doc = doc
            .with_class(0, "container")
            .with_stylesheet(".container { display: flex }");

        compute_styles(&mut doc);

        restyle_subtree(&mut doc, 0, RestyleHint::RESTYLE_SELF);

        let style = doc.computed_style(0).expect("no style");
        assert_eq!(style.display, Display::Flex);
    }

    #[test]
    fn restyle_subtree_descendants() {
        let mut doc = TestDoc::new();
        doc.add_node(0, "div", None);
        doc.add_node(1, "span", Some(0));
        doc.add_node(2, "span", Some(1));
        let mut doc = doc.with_stylesheet("div { color: red }");

        compute_styles(&mut doc);

        // Change stylesheet
        doc.stylist.clear();
        let stylesheet = Stylesheet::parse("div { color: blue }").expect("parse");
        doc.stylist.add_stylesheet(&stylesheet);

        restyle_subtree(
            &mut doc,
            0,
            RestyleHint::RESTYLE_SELF | RestyleHint::RESTYLE_DESCENDANTS,
        );

        let root_style = doc.computed_style(0).expect("no root style");
        assert_eq!(root_style.color, Color::BLUE);

        let child_style = doc.computed_style(1).expect("no child style");
        assert_eq!(child_style.color, Color::BLUE);

        let grandchild_style = doc.computed_style(2).expect("no grandchild style");
        assert_eq!(grandchild_style.color, Color::BLUE);
    }

    #[test]
    fn restyle_empty_hint_does_nothing() {
        let mut doc = TestDoc::new();
        doc.add_node(0, "div", None);

        restyle_subtree(&mut doc, 0, RestyleHint::empty());

        assert!(doc.computed_style(0).is_none());
    }
}
