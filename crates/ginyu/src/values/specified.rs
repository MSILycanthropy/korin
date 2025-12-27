use crate::{UnresolvedValue, macros::keyword_enum};

keyword_enum! {
    pub enum GlobalKeyword {
        Inherit = "inherit",
        Initial = "initial"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Specified<T> {
    Value(T),
    Inherit,
    Initial,
    Unresolved(UnresolvedValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecifiedRef<'a, T> {
    Value(&'a T),
    Inherit,
    Initial,
    Unresolved(&'a UnresolvedValue),
}

impl<T> Specified<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Specified<U> {
        match self {
            Self::Value(v) => Specified::Value(f(v)),
            Self::Inherit => Specified::Inherit,
            Self::Initial => Specified::Initial,
            Self::Unresolved(s) => Specified::Unresolved(s),
        }
    }

    pub const fn as_ref(&self) -> SpecifiedRef<'_, T> {
        match self {
            Self::Value(v) => SpecifiedRef::Value(v),
            Self::Inherit => SpecifiedRef::Inherit,
            Self::Initial => SpecifiedRef::Initial,
            Self::Unresolved(s) => SpecifiedRef::Unresolved(s),
        }
    }

    pub const fn is_unresolved(&self) -> bool {
        matches!(self, Self::Unresolved(_))
    }

    pub const fn as_unresolved(&self) -> Option<&UnresolvedValue> {
        match self {
            Self::Unresolved(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_unresolved_css(&self) -> Option<&str> {
        self.as_unresolved().map(|u| u.css.as_str())
    }
}

impl<T> From<T> for Specified<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

impl<T: Clone> Specified<T> {
    /// Resolve to a concrete value.
    ///
    /// - `Inherit` → use inherited value (or initial if none)
    /// - `Initial` → use initial value
    /// - `Value(v)` → use v
    ///
    /// # Panics
    ///
    /// Will panic if Unresolved
    pub fn resolve(self, inherited: Option<&T>, initial: &T) -> T {
        self.try_resolve(inherited, initial)
            .expect("cannot resolve unresolved value, must `resolve_var` first")
    }

    pub fn try_resolve(self, inherited: Option<&T>, initial: &T) -> Option<T> {
        match self {
            Self::Value(v) => Some(v),
            Self::Inherit => Some(inherited.cloned().unwrap_or_else(|| initial.clone())),
            Self::Initial => Some(initial.clone()),
            Self::Unresolved(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_value_with_vars;

    use super::*;
    use cssparser::{Parser, ParserInput};

    fn make_unresolved(css: &str) -> UnresolvedValue {
        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);
        parse_value_with_vars(&mut parser)
            .expect("parse error")
            .expect("expected var()")
    }

    #[test]
    fn global_keyword() {
        assert_eq!(
            GlobalKeyword::from_name("inherit"),
            Some(GlobalKeyword::Inherit)
        );
        assert_eq!(GlobalKeyword::Initial.to_name(), "initial");
    }

    #[test]
    fn specified_resolve_value() {
        let s: Specified<i32> = Specified::Value(42);
        assert_eq!(s.resolve(Some(&10), &0), 42);
    }

    #[test]
    fn specified_resolve_inherit() {
        let s: Specified<i32> = Specified::Inherit;
        assert_eq!(s.clone().resolve(Some(&10), &0), 10);
        assert_eq!(s.resolve(None, &0), 0);
    }

    #[test]
    fn specified_resolve_initial() {
        let s: Specified<i32> = Specified::Initial;
        assert_eq!(s.resolve(Some(&10), &0), 0);
    }

    #[test]
    fn specified_map() {
        let s: Specified<i32> = Specified::Value(5);
        assert_eq!(s.map(|x| x * 2), Specified::Value(10));

        let s: Specified<i32> = Specified::Inherit;
        assert_eq!(s.map(|x| x * 2), Specified::Inherit);
    }

    #[test]
    fn specified_from_value() {
        let s: Specified<i32> = 42.into();
        assert_eq!(s, Specified::Value(42));
    }

    #[test]
    fn specified_from_unresolved() {
        let unresolved = make_unresolved("var(--x)");
        let s: Specified<i32> = Specified::Unresolved(unresolved);
        assert!(s.is_unresolved());
    }

    #[test]
    fn specified_unresolved_accessors() {
        let unresolved = make_unresolved("var(--primary)");
        let s: Specified<i32> = Specified::Unresolved(unresolved);

        assert!(s.is_unresolved());
        assert!(s.as_unresolved().is_some());
        assert_eq!(s.as_unresolved_css(), Some("var(--primary)"));
    }

    #[test]
    fn specified_try_resolve() {
        let s: Specified<i32> = Specified::Value(42);
        assert_eq!(s.try_resolve(None, &0), Some(42));

        let unresolved = make_unresolved("var(--x)");
        let s: Specified<i32> = Specified::Unresolved(unresolved);
        assert_eq!(s.try_resolve(None, &0), None);
    }

    #[test]
    #[should_panic(expected = "cannot resolve unresolved value")]
    fn specified_resolve_panics_on_unresolved() {
        let unresolved = make_unresolved("var(--x)");
        let s: Specified<i32> = Specified::Unresolved(unresolved);
        s.resolve(None, &0);
    }
}
