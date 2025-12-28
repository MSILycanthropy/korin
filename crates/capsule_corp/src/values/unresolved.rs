use cssparser::TokenSerializationType;
use ginyu_force::Pose;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnresolvedValue {
    pub css: String,

    pub first_token_type: TokenSerializationType,
    pub last_token_type: TokenSerializationType,

    pub references: Vec<VarReference>,
}

impl UnresolvedValue {
    #[must_use]
    pub const fn has_references(&self) -> bool {
        !self.references.is_empty()
    }

    pub fn substitute<'a>(
        &'a self,
        get_var: impl Fn(Pose) -> Option<&'a str>,
    ) -> Result<String, SubstituteError> {
        self.substitute_range(&get_var, 0, self.css.len())
    }

    fn substitute_range<'a>(
        &'a self,
        get_var: &impl Fn(Pose) -> Option<&'a str>,
        range_start: usize,
        range_end: usize,
    ) -> Result<String, SubstituteError> {
        let mut refs_in_range: Vec<&VarReference> = self
            .references
            .iter()
            .filter(|r| r.start >= range_start && r.start < range_end)
            .collect();
        refs_in_range.sort_by_key(|r| r.start);

        let mut result = String::new();
        let mut cursor = range_start;

        for reference in refs_in_range {
            if reference.start < cursor {
                continue;
            }

            result.push_str(&self.css[cursor..reference.start]);

            let value = match get_var(reference.name) {
                Some(v) => v.to_string(),
                None => {
                    if let Some(fallback) = &reference.fallback {
                        let fallback_end = reference.end.saturating_sub(1);
                        self.substitute_range(get_var, fallback.start, fallback_end)?
                    } else {
                        return Err(SubstituteError::UndefinedVariable(
                            reference.name.to_string(),
                        ));
                    }
                }
            };

            result.push_str(&value);
            cursor = reference.end;
        }

        if cursor < range_end {
            result.push_str(&self.css[cursor..range_end]);
        }

        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SubstituteError {
    #[error("undefined variable {0}")]
    UndefinedVariable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarReference {
    pub name: Pose,

    pub start: usize,
    pub end: usize,

    pub prev_token_type: TokenSerializationType,
    pub next_token_type: TokenSerializationType,

    pub fallback: Option<VarFallback>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarFallback {
    pub start: usize,

    pub first_token_type: TokenSerializationType,
    pub last_token_type: TokenSerializationType,
}
