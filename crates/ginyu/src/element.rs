use cssparser::{CowRcStr, ParseError, ParseErrorKind, SourceLocation, ToCss};
use rustc_hash::FxHasher;
use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
};

use precomputed_hash::PrecomputedHash;
use selectors::{
    Parser, SelectorImpl,
    parser::{NonTSPseudoClass, SelectorParseErrorKind},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selectors;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Identifier {
    pub name: String,
    hash: u32,
}

impl Identifier {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let mut hasher = FxHasher::default();
        name.hash(&mut hasher);
        let hash = u32::try_from(hasher.finish()).unwrap_or(u32::MAX);

        Self { name, hash }
    }
}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PrecomputedHash for Identifier {
    fn precomputed_hash(&self) -> u32 {
        self.hash
    }
}

impl Borrow<str> for Identifier {
    fn borrow(&self) -> &str {
        &self.name
    }
}

impl ToCss for Identifier {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: std::fmt::Write,
    {
        write!(dest, "{}", self.name)
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
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
    type BorrowedLocalName = str;
    type BorrowedNamespaceUrl = str;

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
