use selectors::{context::SelectorCaches, matching::matches_selector};

use crate::{
    CapsuleDocument, ConcreteCapsuleElement, SelectorList, bulma::make_context, parse_selector,
};

pub trait QuerySelector: CapsuleDocument {
    fn query_selector(&self, selector: &str) -> Option<Self::NodeId> {
        let selector_list = parse_selector(selector).ok()?;

        self.query_selector_parsed(&selector_list)
    }

    fn query_selector_all(&self, selector: &str) -> Vec<Self::NodeId> {
        let Ok(selector_list) = parse_selector(selector) else {
            return Vec::new();
        };

        self.query_selector_all_parsed(&selector_list)
    }

    fn matches(&self, id: Self::NodeId, selector: &str) -> bool {
        let Ok(selector_list) = parse_selector(selector) else {
            return false;
        };

        self.matches_parsed(id, &selector_list)
    }

    fn query_selector_parsed(&self, selector_list: &SelectorList) -> Option<Self::NodeId> {
        let mut caches = SelectorCaches::default();
        let root = self.root();

        if self.get_element(root).is_some()
            && self.element_matches(root, selector_list, &mut caches)
        {
            return Some(root);
        }

        self.descendants(root).find(|&node| {
            self.get_element(node).is_some()
                && self.element_matches(node, selector_list, &mut caches)
        })
    }

    fn query_selector_all_parsed(&self, selector_list: &SelectorList) -> Vec<Self::NodeId> {
        let mut caches = SelectorCaches::default();
        let mut results = Vec::new();
        let root = self.root();

        if self.get_element(root).is_some()
            && self.element_matches(root, selector_list, &mut caches)
        {
            results.push(root);
        }

        for node in self.descendants(root) {
            if self.get_element(node).is_some()
                && self.element_matches(node, selector_list, &mut caches)
            {
                results.push(node);
            }
        }

        results
    }

    fn matches_parsed(&self, id: Self::NodeId, selector_list: &SelectorList) -> bool {
        let mut caches = SelectorCaches::default();

        self.element_matches(id, selector_list, &mut caches)
    }

    #[doc(hidden)]
    fn element_matches(
        &self,
        id: Self::NodeId,
        selector_list: &SelectorList,
        caches: &mut SelectorCaches,
    ) -> bool {
        let Some(element) = self.get_element(id) else {
            return false;
        };

        let wrapped = ConcreteCapsuleElement::new(element);
        let mut context = make_context(caches);

        for selector in selector_list.slice() {
            if matches_selector(selector, 0, None, &wrapped, &mut context) {
                return true;
            }
        }

        false
    }
}

impl<D: CapsuleDocument> QuerySelector for D {}
