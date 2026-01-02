use crate::view::{AnyView, ChildrenFn, Either, ViewFn};

/// Conditional rendering - shows children when condition is true, fallback otherwise.
///
/// Returns a closure that produces `Either<AnyView, AnyView>` based on the condition.
pub fn show<W>(
    when: W,
    children: ChildrenFn,
    fallback: ViewFn,
) -> impl Fn() -> Either<AnyView, AnyView>
where
    W: Fn() -> bool + 'static,
{
    let children = children;

    move || {
        if when() {
            Either::Left(children())
        } else {
            Either::Right(fallback.call())
        }
    }
}

/// Convenience version without fallback
pub fn show_if<W>(when: W, children: ChildrenFn) -> impl Fn() -> Either<AnyView, AnyView>
where
    W: Fn() -> bool + 'static,
{
    show(when, children, ViewFn::default())
}

/// Convenience version that shows children when condition is false.
pub fn show_unless<W>(when: W, children: ChildrenFn) -> impl Fn() -> Either<AnyView, AnyView>
where
    W: Fn() -> bool + 'static,
{
    show(move || !when(), children, ViewFn::default())
}
