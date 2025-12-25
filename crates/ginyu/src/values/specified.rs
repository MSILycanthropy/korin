use crate::macros::keyword_enum;

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
}

impl<T> Specified<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Specified<U> {
        match self {
            Self::Value(v) => Specified::Value(f(v)),
            Self::Inherit => Specified::Inherit,
            Self::Initial => Specified::Initial,
        }
    }

    pub const fn as_ref(&self) -> Specified<&T> {
        match self {
            Self::Value(v) => Specified::Value(v),
            Self::Inherit => Specified::Inherit,
            Self::Initial => Specified::Initial,
        }
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
    pub fn resolve(self, inherited: Option<&T>, initial: &T) -> T {
        match self {
            Self::Value(v) => v,
            Self::Inherit => inherited.cloned().unwrap_or_else(|| initial.clone()),
            Self::Initial => initial.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(s.resolve(None, &0), 0); // fallback to initial
    }

    #[test]
    fn specified_resolve_initial() {
        let s: Specified<i32> = Specified::Initial;
        assert_eq!(s.resolve(Some(&10), &0), 0);
    }

    #[test]
    fn specified_map() {
        let s: Specified<i32> = Specified::Value(5);
        let doubled = s.map(|x| x * 2);
        assert_eq!(doubled, Specified::Value(10));

        let s: Specified<i32> = Specified::Inherit;
        let mapped = s.map(|x| x * 2);
        assert_eq!(mapped, Specified::Inherit);
    }

    #[test]
    fn specified_from() {
        let s: Specified<i32> = 42.into();
        assert_eq!(s, Specified::Value(42));
    }
}
