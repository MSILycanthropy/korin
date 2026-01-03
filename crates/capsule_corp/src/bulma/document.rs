use selectors::context::SelectorCaches;

use crate::{
    Bulma, CapsuleDocument, ComputedStyle, CustomPropertiesMap, bulma::restyle::RestyleHint,
};

pub fn compute_styles<D: CapsuleDocument>(document: &mut D) {
    let mut stylist = document.take_stylist();
    let mut caches = SelectorCaches::default();
    let root = document.root();

    let children: Vec<_> = document.element_children(root).collect();

    for child in children {
        compute_styles_recursive(document, &mut stylist, &mut caches, child, None, None);
    }

    document.set_stylist(stylist);
}

pub fn restyle_subtree<D: CapsuleDocument>(document: &mut D, node: D::NodeId, hint: RestyleHint) {
    if hint.is_empty() {
        return;
    }

    let mut stylist = document.take_stylist();
    let mut caches = SelectorCaches::default();

    restyle_subtree_inner(document, &mut stylist, &mut caches, node, hint);

    document.set_stylist(stylist);
}

fn restyle_subtree_inner<D: CapsuleDocument>(
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
        let Some(element) = document.get_element(node) else {
            return;
        };
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

fn compute_styles_recursive<D: CapsuleDocument>(
    document: &mut D,
    stylist: &mut Bulma,
    caches: &mut SelectorCaches,
    node: D::NodeId,
    parent_style: Option<&ComputedStyle>,
    parent_custom_properties: Option<&CustomPropertiesMap>,
) {
    let Some(element) = document.get_element(node) else {
        return;
    };

    let (style, custom_properties) =
        stylist.compute_style(&element, parent_style, parent_custom_properties, caches);

    let children: Vec<_> = document.element_children(node).collect();

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

fn restyle_subtree_recursive<D: CapsuleDocument>(
    document: &mut D,
    stylist: &mut Bulma,
    caches: &mut SelectorCaches,
    node: D::NodeId,
    parent_style: Option<&ComputedStyle>,
    parent_custom_properties: Option<&CustomPropertiesMap>,
) {
    let Some(element) = document.get_element(node) else {
        return;
    };
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
