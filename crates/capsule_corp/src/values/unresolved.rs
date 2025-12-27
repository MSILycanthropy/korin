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
        let mut result = String::with_capacity(self.css.len());
        let mut last_end = 0;
        let mut last_token_type = self.first_token_type;

        for reference in &self.references {
            result.push_str(&self.css[last_end..reference.start]);

            let (value, value_first_token) = match get_var(reference.name) {
                Some(value) => (value, reference.prev_token_type),
                None => {
                    if let Some(fallback) = &reference.fallback {
                        let fallback_str = &self.css[fallback.start..reference.end - 1];
                        (fallback_str, fallback.first_token_type)
                    } else {
                        return Err(SubstituteError::UndefinedVariable(
                            reference.name.to_string(),
                        ));
                    }
                }
            };

            if last_token_type.needs_separator_when_before(value_first_token) {
                result.push_str("/**/");
            }

            result.push_str(value);

            last_end = reference.end;
            last_token_type = reference.next_token_type;
        }

        result.push_str(&self.css[last_end..]);

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
