use std::sync::Arc;

use ginyu_force::Pose;
use rustc_hash::{FxHashMap, FxHashSet};
use thiserror::Error;

use crate::{SubstituteError, UnresolvedValue};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CustomPropertiesMap {
    values: Option<Arc<FxHashMap<Pose, String>>>,
}

impl CustomPropertiesMap {
    #[must_use]
    pub const fn new() -> Self {
        Self { values: None }
    }

    pub fn get(&self, name: Pose) -> Option<&str> {
        self.values.as_ref()?.get(&name).map(String::as_str)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.as_ref().is_none_or(|value| value.is_empty())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.as_ref().map_or(0, |value| value.len())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomValue {
    Resolved(String),
    Unresolved(UnresolvedValue),
    Inherit,
    Initial,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResolutionError {
    #[error("cycled detected in property {0}")]
    Cycle(Pose),

    #[error("undefined variable without fallback {0}")]
    Undefined(Pose),

    #[error("substitution error: {0}")]
    Substitution(#[from] SubstituteError),
}

pub struct CustomPropertiesResolver<'a> {
    inherited: Option<&'a CustomPropertiesMap>,
    declarations: Vec<(Pose, CustomValue)>,
}

impl<'a> CustomPropertiesResolver<'a> {
    #[must_use]
    pub const fn new(inherited: Option<&'a CustomPropertiesMap>) -> Self {
        Self {
            inherited,
            declarations: Vec::new(),
        }
    }

    pub fn add(&mut self, name: Pose, value: CustomValue) {
        self.declarations.push((name, value));
    }

    /// Build the resolved custom properties map.
    ///
    /// Resolution order:
    /// 1. Start with inherited values
    /// 2. Apply declarations in cascade order
    /// 3. Resolve `var()` references, detecting cycles
    #[must_use]
    pub fn build(self) -> CustomPropertiesMap {
        if self.declarations.is_empty() {
            return self.inherited.cloned().unwrap_or_default();
        }

        let mut values: FxHashMap<Pose, String> = self
            .inherited
            .and_then(|i| i.values.as_ref())
            .map(|v| v.as_ref().clone())
            .unwrap_or_default();

        let mut pending: FxHashMap<Pose, CustomValue> = FxHashMap::default();

        for (name, value) in self.declarations {
            pending.insert(name, value);
        }

        let mut resolving: FxHashSet<Pose> = FxHashSet::default();

        for (name, value) in &pending {
            let _ = resolve_property(
                *name,
                value,
                &pending,
                &mut values,
                &mut resolving,
                self.inherited,
            );
        }

        if values.is_empty() {
            CustomPropertiesMap { values: None }
        } else {
            CustomPropertiesMap {
                values: Some(Arc::new(values)),
            }
        }
    }
}

fn resolve_property(
    name: Pose,
    value: &CustomValue,
    pending: &FxHashMap<Pose, CustomValue>,
    resolved: &mut FxHashMap<Pose, String>,
    resolving: &mut FxHashSet<Pose>,
    inherited: Option<&CustomPropertiesMap>,
) -> Result<(), ResolutionError> {
    match value {
        CustomValue::Initial => {
            resolved.remove(&name);
            return Ok(());
        }
        CustomValue::Inherit => return Ok(()),
        _ => {}
    }

    if !resolving.insert(name) {
        resolved.remove(&name);
        return Err(ResolutionError::Cycle(name));
    }

    let result = match value {
        CustomValue::Resolved(str) => {
            resolved.insert(name, str.clone());
            Ok(())
        }
        CustomValue::Unresolved(unresolved) => {
            for reference in &unresolved.references {
                if !resolved.contains_key(&reference.name)
                    && let Some(dep_value) = pending.get(&reference.name)
                {
                    let result = resolve_property(
                        reference.name,
                        dep_value,
                        pending,
                        resolved,
                        resolving,
                        inherited,
                    );

                    if result.is_err() {
                        resolved.remove(&name);
                        resolving.remove(&name);
                        return result;
                    }
                }
            }

            match unresolved.substitute(|dep_name| resolved.get(&dep_name).map(String::as_str)) {
                Ok(substituted) => {
                    resolved.insert(name, substituted);
                    Ok(())
                }
                Err(err) => {
                    if inherited.and_then(|i| i.get(name)).is_none() {
                        resolved.remove(&name);
                    }

                    Err(err.into())
                }
            }
        }
        CustomValue::Inherit | CustomValue::Initial => unreachable!(),
    };

    resolving.remove(&name);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_value_with_vars;
    use cssparser::{Parser, ParserInput};
    use ginyu_force::Pose;

    fn make_unresolved(css: &str) -> UnresolvedValue {
        let mut input = ParserInput::new(css);
        let mut parser = Parser::new(&mut input);
        parse_value_with_vars(&mut parser)
            .expect("parse error")
            .expect("expected var()")
    }

    #[test]
    fn empty_builder() {
        let map = CustomPropertiesResolver::new(None).build();
        assert!(map.is_empty());
    }

    #[test]
    fn simple_resolved() {
        let mut builder = CustomPropertiesResolver::new(None);
        builder.add(Pose::from("color"), CustomValue::Resolved("red".into()));

        let map = builder.build();
        assert_eq!(map.get(Pose::from("color")), Some("red"));
    }

    #[test]
    fn cascade_order() {
        let mut builder = CustomPropertiesResolver::new(None);
        builder.add(Pose::from("color"), CustomValue::Resolved("red".into()));
        builder.add(Pose::from("color"), CustomValue::Resolved("blue".into()));

        let map = builder.build();
        assert_eq!(map.get(Pose::from("color")), Some("blue"));
    }

    #[test]
    fn inheritance() {
        let mut parent_builder = CustomPropertiesResolver::new(None);
        parent_builder.add(Pose::from("color"), CustomValue::Resolved("red".into()));
        let parent = parent_builder.build();

        let child_builder = CustomPropertiesResolver::new(Some(&parent));
        let child = child_builder.build();

        assert_eq!(child.get(Pose::from("color")), Some("red"));
    }

    #[test]
    fn override_inherited() {
        let mut parent_builder = CustomPropertiesResolver::new(None);
        parent_builder.add(Pose::from("color"), CustomValue::Resolved("red".into()));
        let parent = parent_builder.build();

        let mut child_builder = CustomPropertiesResolver::new(Some(&parent));
        child_builder.add(Pose::from("color"), CustomValue::Resolved("blue".into()));
        let child = child_builder.build();

        assert_eq!(child.get(Pose::from("color")), Some("blue"));
    }

    #[test]
    fn initial_resets() {
        let mut parent_builder = CustomPropertiesResolver::new(None);
        parent_builder.add(Pose::from("color"), CustomValue::Resolved("red".into()));
        let parent = parent_builder.build();

        let mut child_builder = CustomPropertiesResolver::new(Some(&parent));
        child_builder.add(Pose::from("color"), CustomValue::Initial);
        let child = child_builder.build();

        assert_eq!(child.get(Pose::from("color")), None);
    }

    #[test]
    fn var_substitution() {
        let mut builder = CustomPropertiesResolver::new(None);
        builder.add(Pose::from("primary"), CustomValue::Resolved("blue".into()));
        builder.add(
            Pose::from("color"),
            CustomValue::Unresolved(make_unresolved("var(--primary)")),
        );

        let map = builder.build();
        assert_eq!(map.get(Pose::from("color")), Some("blue"));
    }

    #[test]
    fn var_chain() {
        let mut builder = CustomPropertiesResolver::new(None);
        builder.add(Pose::from("a"), CustomValue::Resolved("red".into()));
        builder.add(
            Pose::from("b"),
            CustomValue::Unresolved(make_unresolved("var(--a)")),
        );
        builder.add(
            Pose::from("c"),
            CustomValue::Unresolved(make_unresolved("var(--b)")),
        );

        let map = builder.build();
        assert_eq!(map.get(Pose::from("c")), Some("red"));
    }

    #[test]
    fn cycle_detection() {
        let mut builder = CustomPropertiesResolver::new(None);
        builder.add(
            Pose::from("a"),
            CustomValue::Unresolved(make_unresolved("var(--b)")),
        );
        builder.add(
            Pose::from("b"),
            CustomValue::Unresolved(make_unresolved("var(--a)")),
        );

        let map = builder.build();
        // Both should be invalid (removed)
        assert_eq!(map.get(Pose::from("a")), None);
        assert_eq!(map.get(Pose::from("b")), None);
    }

    #[test]
    fn undefined_with_fallback() {
        let mut builder = CustomPropertiesResolver::new(None);
        builder.add(
            Pose::from("color"),
            CustomValue::Unresolved(make_unresolved("var(--missing, red)")),
        );

        let map = builder.build();
        assert_eq!(map.get(Pose::from("color")), Some("red"));
    }

    #[test]
    fn undefined_no_fallback_inherits() {
        let mut parent_builder = CustomPropertiesResolver::new(None);
        parent_builder.add(
            Pose::from("color"),
            CustomValue::Resolved("inherited-red".into()),
        );
        let parent = parent_builder.build();

        let mut child_builder = CustomPropertiesResolver::new(Some(&parent));
        child_builder.add(
            Pose::from("color"),
            CustomValue::Unresolved(make_unresolved("var(--missing)")),
        );
        let child = child_builder.build();

        // Falls back to inherited value
        assert_eq!(child.get(Pose::from("color")), Some("inherited-red"));
    }
}
