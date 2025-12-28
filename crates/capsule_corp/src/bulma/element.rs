use cssparser::{CowRcStr, ParseError, ParseErrorKind, SourceLocation, ToCss};
use ginyu_force::Pose;
use precomputed_hash::PrecomputedHash;
use selectors::{
    Element, OpaqueElement, Parser, SelectorImpl,
    attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint},
    bloom::BloomFilter,
    context::MatchingContext,
    matching::ElementSelectorFlags,
    parser::{NonTSPseudoClass, SelectorParseErrorKind},
};
use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
    ops::DerefMut,
};
use std::{fmt::Debug, ops::Deref};

use crate::ElementState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selectors;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Identifier(Pose);

impl Identifier {
    pub fn new(name: impl Into<Pose>) -> Self {
        Self(name.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    #[must_use]
    pub const fn as_pose(&self) -> Pose {
        self.0
    }
}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state);
    }
}

impl PrecomputedHash for Identifier {
    #[allow(clippy::cast_possible_truncation)]
    fn precomputed_hash(&self) -> u32 {
        let mut hasher = rustc_hash::FxHasher::default();
        self.0.as_str().hash(&mut hasher);
        hasher.finish() as u32
    }
}

impl Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl ToCss for Identifier {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(dest, "{}", self.0)
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<Pose> for Identifier {
    fn from(value: Pose) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PseudoClass {
    Hover,
    Focus,
    Active,
    Disabled,
    Checked,
    FirstChild,
    LastChild,
    NthChild(i32),
}

impl ToCss for PseudoClass {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        match self {
            Self::Hover => write!(dest, ":hover"),
            Self::Focus => write!(dest, ":focus"),
            Self::Active => write!(dest, ":active"),
            Self::Disabled => write!(dest, ":disabled"),
            Self::Checked => write!(dest, ":checked"),
            Self::FirstChild => write!(dest, ":first-child"),
            Self::LastChild => write!(dest, ":last-child"),
            Self::NthChild(n) => write!(dest, ":nth-child({n})"),
        }
    }
}

impl NonTSPseudoClass for PseudoClass {
    type Impl = Selectors;

    fn is_active_or_hover(&self) -> bool {
        matches!(self, Self::Hover | Self::Active)
    }

    fn is_user_action_state(&self) -> bool {
        matches!(self, Self::Hover | Self::Active | Self::Focus)
    }
}

// We don't support these technically
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PseudoElement;

impl ToCss for PseudoElement {
    fn to_css<W>(&self, _dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        Ok(())
    }
}

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = Selectors;
}

impl SelectorImpl for Selectors {
    type AttrValue = Identifier;
    type Identifier = Identifier;
    type LocalName = Identifier;
    type NamespacePrefix = Identifier;
    type NamespaceUrl = Identifier;
    type BorrowedLocalName = Identifier;
    type BorrowedNamespaceUrl = Identifier;

    type NonTSPseudoClass = PseudoClass;
    type PseudoElement = PseudoElement;

    type ExtraMatchingData<'a> = ();
}

pub struct SelectorParser;

impl<'i> Parser<'i> for SelectorParser {
    type Impl = Selectors;
    type Error = SelectorParseErrorKind<'i>;

    fn parse_parent_selector(&self) -> bool {
        true
    }

    fn parse_non_ts_pseudo_class(
        &self,
        location: SourceLocation,
        name: CowRcStr<'i>,
    ) -> Result<<Self::Impl as SelectorImpl>::NonTSPseudoClass, ParseError<'i, Self::Error>> {
        match name.as_ref() {
            "hover" => Ok(PseudoClass::Hover),
            "focus" => Ok(PseudoClass::Focus),
            "active" => Ok(PseudoClass::Active),
            "disabled" => Ok(PseudoClass::Disabled),
            "checked" => Ok(PseudoClass::Checked),
            "first-child" => Ok(PseudoClass::FirstChild),
            "last-child" => Ok(PseudoClass::LastChild),
            _ => Err(ParseError {
                kind: ParseErrorKind::Custom(
                    SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name),
                ),
                location,
            }),
        }
    }

    fn parse_non_ts_functional_pseudo_class<'t>(
        &self,
        name: CowRcStr<'i>,
        parser: &mut cssparser::Parser<'i, 't>,
        _after_part: bool,
    ) -> Result<<Self::Impl as SelectorImpl>::NonTSPseudoClass, ParseError<'i, Self::Error>> {
        let location = parser.current_source_location();
        match name.as_ref() {
            "nth-child" => {
                let n = parser.expect_integer()?;
                Ok(PseudoClass::NthChild(n))
            }
            _ => Err(cssparser::ParseError {
                kind: cssparser::ParseErrorKind::Custom(
                    SelectorParseErrorKind::UnsupportedPseudoClassOrElement(name),
                ),
                location,
            }),
        }
    }
}

pub trait TElement: Sized + Clone + Debug + PartialEq {
    fn tag_name(&self) -> Pose;

    fn id(&self) -> Option<Pose>;

    fn has_class(&self, name: &str) -> bool;

    fn each_class<F: FnMut(Pose)>(&self, callback: F);

    fn get_attribute(&self, name: Pose) -> Option<&str>;

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

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct CapsuleElement<E>(pub E);

impl<E> CapsuleElement<E> {
    pub const fn new(element: E) -> Self {
        Self(element)
    }

    pub fn into_inner(self) -> E {
        self.0
    }
}

impl<E> Deref for CapsuleElement<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for CapsuleElement<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E: TElement> Element for CapsuleElement<E> {
    type Impl = Selectors;

    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self)
    }

    fn parent_element(&self) -> Option<Self> {
        self.parent().map(CapsuleElement)
    }

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    fn is_pseudo_element(&self) -> bool {
        false
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        self.prev_sibling().map(CapsuleElement)
    }

    fn next_sibling_element(&self) -> Option<Self> {
        self.next_sibling().map(CapsuleElement)
    }

    fn first_element_child(&self) -> Option<Self> {
        None
    }

    fn is_html_element_in_html_document(&self) -> bool {
        true
    }

    fn has_local_name(&self, local_name: &Identifier) -> bool {
        self.tag_name() == local_name.as_pose()
    }

    fn has_namespace(&self, _ns: &Identifier) -> bool {
        true
    }

    fn is_part(&self, _name: &Identifier) -> bool {
        false
    }

    fn imported_part(&self, _name: &Identifier) -> Option<Identifier> {
        None
    }

    fn is_same_type(&self, other: &Self) -> bool {
        self.tag_name() == other.tag_name()
    }

    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&Identifier>,
        local_name: &<Self::Impl as SelectorImpl>::LocalName,
        operation: &AttrSelectorOperation<&Identifier>,
    ) -> bool {
        match ns {
            NamespaceConstraint::Any => {}
            NamespaceConstraint::Specific(id) if id.as_str().is_empty() => {}
            NamespaceConstraint::Specific(_) => return false,
        }

        let Some(value) = self.get_attribute(local_name.as_pose()) else {
            return false;
        };

        operation.eval_str(value)
    }

    // TODO: Use the `context` for perf improvements
    #[allow(clippy::cast_sign_loss)]
    fn match_non_ts_pseudo_class(
        &self,
        pseudo_class: &PseudoClass,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        use PseudoClass::*;
        let state = self.state();

        match pseudo_class {
            Hover => state.contains(ElementState::HOVER),
            Focus => state.contains(ElementState::FOCUS),
            Active => state.contains(ElementState::ACTIVE),
            Disabled => state.contains(ElementState::DISABLED),
            Checked => state.contains(ElementState::CHECKED),
            FirstChild => self.is_first_child(),
            LastChild => self.is_last_child(),
            NthChild(n) => self.sibling_index() == (*n as usize),
        }
    }

    fn match_pseudo_element(
        &self,
        _pe: &PseudoElement,
        _context: &mut MatchingContext<Self::Impl>,
    ) -> bool {
        false
    }

    fn apply_selector_flags(&self, _flags: ElementSelectorFlags) {}

    fn has_custom_state(&self, _name: &Identifier) -> bool {
        false
    }

    // TODO: Links
    fn is_link(&self) -> bool {
        false
    }

    fn is_html_slot_element(&self) -> bool {
        false
    }

    fn has_id(&self, id: &Identifier, _case_sensitivity: CaseSensitivity) -> bool {
        self.id().is_some_and(|pose_id| pose_id == id.as_pose())
    }

    fn has_class(&self, name: &Identifier, _case_sensitivity: CaseSensitivity) -> bool {
        self.0.has_class(name.as_str())
    }

    fn is_empty(&self) -> bool {
        !self.has_children()
    }

    fn is_root(&self) -> bool {
        self.parent().is_none()
    }

    fn add_element_unique_hashes(&self, _filter: &mut BloomFilter) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_from_str() {
        let id = Identifier::from("test");
        assert_eq!(id.as_str(), "test");
    }

    #[test]
    fn identifier_from_pose() {
        let pose = Pose::from("test");
        let id = Identifier::from(pose);
        assert_eq!(id.as_pose(), pose);
    }

    #[test]
    fn identifier_equality() {
        let a = Identifier::from("test");
        let b = Identifier::from("test");
        assert_eq!(a, b);
    }

    #[test]
    fn identifier_borrow() {
        let id = Identifier::from("test");
        let s: &str = id.borrow();
        assert_eq!(s, "test");
    }

    #[test]
    fn identifier_to_css() {
        let id = Identifier::from("my-class");
        let mut out = String::new();
        id.to_css(&mut out).expect("to_css failed");
        assert_eq!(out, "my-class");
    }
}
