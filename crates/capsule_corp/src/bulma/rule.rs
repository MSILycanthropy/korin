use std::sync::Arc;

use selectors::parser::Selector;

use crate::{Selectors, parser::Declaration};

#[derive(Debug, Clone)]
pub struct BulmaRule {
    pub selector: Selector<Selectors>,
    pub declarations: Arc<Vec<Declaration>>,
    pub source_order: u32,
}

impl BulmaRule {
    pub const fn new(
        selector: Selector<Selectors>,
        declarations: Arc<Vec<Declaration>>,
        source_order: u32,
    ) -> Self {
        Self {
            selector,
            declarations,
            source_order,
        }
    }

    #[inline]
    pub fn specificity(&self) -> u32 {
        self.selector.specificity()
    }
}
