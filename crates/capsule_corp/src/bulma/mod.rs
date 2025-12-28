mod cascade;
mod computed;
mod core;
mod element;
mod invalidation;
mod restyle;
mod rule;

pub use computed::*;
pub use core::*;
pub use element::*;
use selectors::context::{
    MatchingContext, MatchingForInvalidation, MatchingMode, NeedsSelectorFlags, QuirksMode,
    SelectorCaches,
};

fn make_context(caches: &mut SelectorCaches) -> MatchingContext<'_, Selectors> {
    MatchingContext::new(
        MatchingMode::Normal,
        None,
        caches,
        QuirksMode::NoQuirks,
        NeedsSelectorFlags::No,
        MatchingForInvalidation::No,
    )
}
