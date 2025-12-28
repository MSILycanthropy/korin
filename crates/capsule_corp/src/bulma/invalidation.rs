use ginyu_force::Pose;
use rustc_hash::FxHashMap;
use selectors::parser::{Combinator, Component, Selector};
use smallvec::SmallVec;

use crate::{ElementState, PseudoClass, Selectors, bulma::restyle::RestyleHint};

#[derive(Debug, Default)]
pub struct InvalidationMap {
    state: FxHashMap<ElementState, SmallVec<[Dependency; 4]>>,
    attribute: FxHashMap<Pose, SmallVec<[Dependency; 4]>>,
    class: FxHashMap<Pose, SmallVec<[Dependency; 4]>>,
    id: FxHashMap<Pose, SmallVec<[Dependency; 4]>>,
}

impl InvalidationMap {
    #[allow(unused, reason = "tests")]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.state.clear();
        self.attribute.clear();
        self.class.clear();
        self.id.clear();
    }

    pub fn shrink_to_fit(&mut self) {
        self.state.shrink_to_fit();
        self.attribute.shrink_to_fit();
        self.class.shrink_to_fit();
        self.id.shrink_to_fit();
    }

    pub fn register_selector(&mut self, selector: &Selector<Selectors>) {
        let mut location = DependencyLocation::Subject;
        let mut components = selector.iter();

        loop {
            for component in &mut components {
                self.register_component(component, location);
            }

            match components.next_sequence() {
                None => break,
                Some(combinator) => {
                    location = match combinator {
                        Combinator::Child | Combinator::Descendant => DependencyLocation::Ancestor,
                        Combinator::NextSibling | Combinator::LaterSibling => {
                            DependencyLocation::PreviousSibling
                        }
                        _ => location,
                    }
                }
            }
        }
    }

    fn register_component(
        &mut self,
        component: &Component<Selectors>,
        location: DependencyLocation,
    ) {
        use Component::*;

        let dependency = Dependency { location };

        match component {
            ID(id) => {
                self.id.entry(id.as_pose()).or_default().push(dependency);
            }
            Class(class) => {
                self.class
                    .entry(class.as_pose())
                    .or_default()
                    .push(dependency);
            }
            AttributeInNoNamespaceExists { local_name, .. }
            | AttributeInNoNamespace { local_name, .. } => {
                self.attribute
                    .entry(local_name.as_pose())
                    .or_default()
                    .push(dependency);
            }
            NonTSPseudoClass(pseudo) => {
                let state = pseudo_class_to_state(pseudo);
                if !state.is_empty() {
                    self.state.entry(state).or_default().push(dependency);
                }
            }
            _ => {}
        }
    }

    pub fn restyle_hint_for_state_change(
        &self,
        old: ElementState,
        new: ElementState,
    ) -> RestyleHint {
        let changed = old ^ new;
        let mut hint = RestyleHint::empty();

        for (state, dependencies) in &self.state {
            if changed.intersects(*state) {
                for dependency in dependencies {
                    hint |= dependency.location.to_restyle_hint();
                }
            }
        }

        hint
    }

    pub fn restyle_hint_for_attribute_change(&self, attribute: Pose) -> RestyleHint {
        let mut hint = RestyleHint::empty();

        if let Some(dependencies) = self.attribute.get(&attribute) {
            for dependency in dependencies {
                hint |= dependency.location.to_restyle_hint();
            }
        }

        hint
    }

    pub fn restyle_hint_for_class_change(&self, class: Pose) -> RestyleHint {
        let mut hint = RestyleHint::empty();

        if let Some(dependencies) = self.class.get(&class) {
            for dependency in dependencies {
                hint |= dependency.location.to_restyle_hint();
            }
        }

        hint
    }

    pub fn restyle_hint_for_id_change(&self, id: Pose) -> RestyleHint {
        let mut hint = RestyleHint::empty();

        if let Some(dependencies) = self.id.get(&id) {
            for dependency in dependencies {
                hint |= dependency.location.to_restyle_hint();
            }
        }

        hint
    }

    #[inline]
    pub fn has_state_dependency(&self, state: ElementState) -> bool {
        self.state.keys().any(|s| s.intersects(state))
    }

    #[inline]
    pub fn has_attribute_dependency(&self, attribute: Pose) -> bool {
        self.attribute.contains_key(&attribute)
    }

    #[inline]
    #[allow(unused, reason = "tests")]
    pub fn has_class_dependency(&self, class: Pose) -> bool {
        self.class.contains_key(&class)
    }

    #[inline]
    #[allow(unused, reason = "tests")]
    pub fn has_id_dependency(&self, id: Pose) -> bool {
        self.id.contains_key(&id)
    }
}

const fn pseudo_class_to_state(pseudo: &PseudoClass) -> ElementState {
    match pseudo {
        PseudoClass::Hover => ElementState::HOVER,
        PseudoClass::Focus => ElementState::FOCUS,
        PseudoClass::Active => ElementState::ACTIVE,
        PseudoClass::Disabled => ElementState::DISABLED,
        PseudoClass::Checked => ElementState::CHECKED,
        PseudoClass::FirstChild | PseudoClass::LastChild | PseudoClass::NthChild(_) => {
            ElementState::empty()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub location: DependencyLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyLocation {
    Subject,
    Ancestor,
    PreviousSibling,
}

impl DependencyLocation {
    pub const fn to_restyle_hint(self) -> RestyleHint {
        match self {
            Self::Subject => RestyleHint::RESTYLE_SELF,
            Self::Ancestor => RestyleHint::RESTYLE_DESCENDANTS,
            Self::PreviousSibling => RestyleHint::RESTYLE_LATER_SIBLINGS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SelectorParser;
    use cssparser::ParserInput;

    fn parse_selector(s: &str) -> Selector<Selectors> {
        let mut input = ParserInput::new(s);
        let mut parser = cssparser::Parser::new(&mut input);
        Selector::parse(&SelectorParser, &mut parser).expect("failed to parse selector")
    }

    #[test]
    fn empty_map() {
        let map = InvalidationMap::new();
        assert!(!map.has_state_dependency(ElementState::HOVER));
        assert!(!map.has_class_dependency(Pose::from("foo")));
        assert!(!map.has_id_dependency(Pose::from("bar")));
        assert!(!map.has_attribute_dependency(Pose::from("disabled")));
    }

    #[test]
    fn register_hover_selector() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector(".btn:hover"));

        assert!(map.has_state_dependency(ElementState::HOVER));
        assert!(map.has_class_dependency(Pose::from("btn")));
    }

    #[test]
    fn register_focus_selector() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector("input:focus"));

        assert!(map.has_state_dependency(ElementState::FOCUS));
    }

    #[test]
    fn register_id_selector() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector("#main"));

        assert!(map.has_id_dependency(Pose::from("main")));
    }

    #[test]
    fn register_attribute_selector() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector("[disabled]"));

        assert!(map.has_attribute_dependency(Pose::from("disabled")));
    }

    #[test]
    fn state_change_restyle_hint_subject() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector(".btn:hover"));

        let hint = map.restyle_hint_for_state_change(ElementState::empty(), ElementState::HOVER);

        assert!(hint.contains(RestyleHint::RESTYLE_SELF));
    }

    #[test]
    fn state_change_no_hint_when_unrelated() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector(".btn:hover"));

        // Focus changed, but we only have hover dependency
        let hint = map.restyle_hint_for_state_change(ElementState::empty(), ElementState::FOCUS);

        assert!(hint.is_empty());
    }

    #[test]
    fn class_change_restyle_hint() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector(".active"));

        let hint = map.restyle_hint_for_class_change(Pose::from("active"));
        assert!(hint.contains(RestyleHint::RESTYLE_SELF));
    }

    #[test]
    fn class_change_no_hint_when_unrelated() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector(".active"));

        let hint = map.restyle_hint_for_class_change(Pose::from("inactive"));
        assert!(hint.is_empty());
    }

    #[test]
    fn descendant_combinator_gives_descendants_hint() {
        let mut map = InvalidationMap::new();
        // .parent:hover .child - hover is on ancestor
        map.register_selector(&parse_selector(".parent:hover .child"));

        let hint = map.restyle_hint_for_state_change(ElementState::empty(), ElementState::HOVER);

        assert!(hint.contains(RestyleHint::RESTYLE_DESCENDANTS));
    }

    #[test]
    fn child_combinator_gives_descendants_hint() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector(".parent:hover > .child"));

        let hint = map.restyle_hint_for_state_change(ElementState::empty(), ElementState::HOVER);

        assert!(hint.contains(RestyleHint::RESTYLE_DESCENDANTS));
    }

    #[test]
    fn sibling_combinator_gives_siblings_hint() {
        let mut map = InvalidationMap::new();
        // .prev:hover + .next - hover is on previous sibling
        map.register_selector(&parse_selector(".prev:hover + .next"));

        let hint = map.restyle_hint_for_state_change(ElementState::empty(), ElementState::HOVER);

        assert!(hint.contains(RestyleHint::RESTYLE_LATER_SIBLINGS));
    }

    #[test]
    fn later_sibling_combinator_gives_siblings_hint() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector(".prev:hover ~ .next"));

        let hint = map.restyle_hint_for_state_change(ElementState::empty(), ElementState::HOVER);

        assert!(hint.contains(RestyleHint::RESTYLE_LATER_SIBLINGS));
    }

    #[test]
    fn structural_pseudo_no_state_dependency() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector(":first-child"));

        // Structural pseudo-classes don't map to ElementState
        assert!(!map.has_state_dependency(ElementState::HOVER));
        assert!(!map.has_state_dependency(ElementState::FOCUS));
    }

    #[test]
    fn clear_removes_all_dependencies() {
        let mut map = InvalidationMap::new();
        map.register_selector(&parse_selector(".btn:hover"));
        map.register_selector(&parse_selector("#main"));
        map.register_selector(&parse_selector("[disabled]"));

        map.clear();

        assert!(!map.has_state_dependency(ElementState::HOVER));
        assert!(!map.has_class_dependency(Pose::from("btn")));
        assert!(!map.has_id_dependency(Pose::from("main")));
        assert!(!map.has_attribute_dependency(Pose::from("disabled")));
    }
}
