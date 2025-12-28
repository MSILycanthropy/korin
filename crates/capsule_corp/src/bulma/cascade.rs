use ginyu_force::Pose;
use rustc_hash::FxHashMap;
use selectors::parser::Selector;
use smallvec::SmallVec;

use crate::{Selectors, bulma::rule::BulmaRule};

#[derive(Debug, Default)]
pub struct CascadeData {
    rules_by_id: FxHashMap<Pose, SmallVec<[BulmaRule; 4]>>,
    rules_by_class: FxHashMap<Pose, SmallVec<[BulmaRule; 4]>>,
    rules_by_tag: FxHashMap<Pose, SmallVec<[BulmaRule; 4]>>,
    universal_rules: Vec<BulmaRule>,

    pub num_selectors: usize,
    pub num_declarations: usize,
}

impl CascadeData {
    #[allow(unused, reason = "tests")]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get rules that might match an element with the given ID.
    #[inline]
    pub fn rules_by_id(&self, id: Pose) -> Option<&SmallVec<[BulmaRule; 4]>> {
        self.rules_by_id.get(&id)
    }

    /// Get rules that might match an element with the given class.
    #[inline]
    pub fn rules_by_class(&self, class: Pose) -> Option<&SmallVec<[BulmaRule; 4]>> {
        self.rules_by_class.get(&class)
    }

    /// Get rules that might match an element with the given tag name.
    #[inline]
    pub fn rules_by_tag(&self, tag: Pose) -> Option<&SmallVec<[BulmaRule; 4]>> {
        self.rules_by_tag.get(&tag)
    }

    /// Get rules that need to be checked for all elements.
    #[inline]
    pub fn universal_rules(&self) -> &[BulmaRule] {
        &self.universal_rules
    }

    pub fn insert(&mut self, rule: BulmaRule) {
        use BucketKey::*;

        self.num_selectors += 1;
        self.num_declarations += rule.declarations.len();

        match extract_bucket_key(&rule.selector) {
            Id(id) => self.rules_by_id.entry(id).or_default().push(rule),
            Class(class) => self.rules_by_class.entry(class).or_default().push(rule),
            Tag(tag) => self.rules_by_tag.entry(tag).or_default().push(rule),
            Universal => self.universal_rules.push(rule),
        }
    }

    pub fn clear(&mut self) {
        self.rules_by_id.clear();
        self.rules_by_class.clear();
        self.rules_by_tag.clear();
        self.universal_rules.clear();

        self.num_declarations = 0;
        self.num_selectors = 0;
    }

    pub fn shrink_to_fit(&mut self) {
        self.rules_by_id.shrink_to_fit();
        self.rules_by_class.shrink_to_fit();
        self.rules_by_tag.shrink_to_fit();
        self.universal_rules.shrink_to_fit();
    }
}

fn extract_bucket_key(selector: &Selector<Selectors>) -> BucketKey {
    use selectors::parser::Component::*;

    let mut class_key: Option<Pose> = None;
    let mut tag_key: Option<Pose> = None;

    for component in selector.iter() {
        match component {
            ID(id) => return BucketKey::Id(id.as_pose()),
            Class(class) if class_key.is_none() => class_key = Some(class.as_pose()),
            LocalName(local) if tag_key.is_none() => tag_key = Some(local.name.as_pose()),
            _ => {}
        }
    }

    class_key.map_or_else(
        || tag_key.map_or_else(|| BucketKey::Universal, BucketKey::Tag),
        BucketKey::Class,
    )
}

enum BucketKey {
    Id(Pose),
    Class(Pose),
    Tag(Pose),
    Universal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn parse_selector(s: &str) -> Selector<Selectors> {
        use crate::SelectorParser;
        use cssparser::ParserInput;

        let mut input = ParserInput::new(s);
        let mut parser = cssparser::Parser::new(&mut input);
        Selector::parse(&SelectorParser, &mut parser).expect("failed to parse selector")
    }

    fn make_rule(selector: &str, source_order: u32) -> BulmaRule {
        BulmaRule::new(parse_selector(selector), Arc::new(vec![]), source_order)
    }

    #[test]
    fn empty_cascade_data() {
        let data = CascadeData::new();
        assert_eq!(data.num_selectors, 0);
        assert_eq!(data.num_declarations, 0);
        assert!(data.universal_rules().is_empty());
    }

    #[test]
    fn insert_id_selector() {
        let mut data = CascadeData::new();
        data.insert(make_rule("#main", 0));

        assert_eq!(data.num_selectors, 1);
        assert!(data.rules_by_id(Pose::from("main")).is_some());
        assert!(data.rules_by_class(Pose::from("main")).is_none());
        assert!(data.universal_rules().is_empty());
    }

    #[test]
    fn insert_class_selector() {
        let mut data = CascadeData::new();
        data.insert(make_rule(".container", 0));

        assert_eq!(data.num_selectors, 1);
        assert!(data.rules_by_class(Pose::from("container")).is_some());
        assert!(data.rules_by_id(Pose::from("container")).is_none());
        assert!(data.universal_rules().is_empty());
    }

    #[test]
    fn insert_tag_selector() {
        let mut data = CascadeData::new();
        data.insert(make_rule("div", 0));

        assert_eq!(data.num_selectors, 1);
        assert!(data.rules_by_tag(Pose::from("div")).is_some());
        assert!(data.universal_rules().is_empty());
    }

    #[test]
    fn insert_universal_selector() {
        let mut data = CascadeData::new();
        data.insert(make_rule("*", 0));

        assert_eq!(data.num_selectors, 1);
        assert_eq!(data.universal_rules().len(), 1);
    }

    #[test]
    fn insert_attribute_selector_is_universal() {
        let mut data = CascadeData::new();
        data.insert(make_rule("[disabled]", 0));

        // Attribute-only selectors go to universal bucket
        assert_eq!(data.universal_rules().len(), 1);
    }

    #[test]
    fn id_takes_priority_over_class() {
        let mut data = CascadeData::new();
        data.insert(make_rule("#foo.bar", 0));

        // Should be bucketed by ID, not class
        assert!(data.rules_by_id(Pose::from("foo")).is_some());
        assert!(data.rules_by_class(Pose::from("bar")).is_none());
    }

    #[test]
    fn class_takes_priority_over_tag() {
        let mut data = CascadeData::new();
        data.insert(make_rule("div.container", 0));

        // Should be bucketed by class, not tag
        assert!(data.rules_by_class(Pose::from("container")).is_some());
        assert!(data.rules_by_tag(Pose::from("div")).is_none());
    }

    #[test]
    fn multiple_rules_same_bucket() {
        let mut data = CascadeData::new();
        data.insert(make_rule(".btn", 0));
        data.insert(make_rule(".btn:hover", 1));

        let rules = data.rules_by_class(Pose::from("btn")).expect("failed");
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn clear_removes_all() {
        let mut data = CascadeData::new();
        data.insert(make_rule("#id", 0));
        data.insert(make_rule(".class", 1));
        data.insert(make_rule("tag", 2));
        data.insert(make_rule("*", 3));

        data.clear();

        assert!(data.rules_by_id(Pose::from("id")).is_none());
        assert!(data.rules_by_class(Pose::from("class")).is_none());
        assert!(data.rules_by_tag(Pose::from("tag")).is_none());
        assert!(data.universal_rules().is_empty());
    }
}
