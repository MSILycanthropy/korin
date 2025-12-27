use cssparser::TokenSerializationType;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarReference {
    pub name: String,

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
