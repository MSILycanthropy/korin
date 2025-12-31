use std::rc::Rc;

use crate::view::{
    AnyViewState, View,
    any_view::AnyView,
    context::{BuildContext, RebuildContext},
};

/// Children that can only be called once. Most common type for component children.
pub type Children = Box<dyn FnOnce() -> AnyView>;

/// Children that can be called multiple times (immutable).
pub type ChildrenFn = Rc<dyn Fn() -> AnyView>;

/// Children that can be called multiple times (mutable).
pub type ChildrenFnMut = Box<dyn FnMut() -> AnyView>;

/// Newtype wrapper for a function that returns a view.
/// Useful for optional props like fallback in `Show`.
pub struct ViewFn(ChildrenFn);

impl ViewFn {
    pub fn new<F, V>(f: F) -> Self
    where
        F: Fn() -> V + 'static,
        V: View + 'static,
        V::State: 'static,
    {
        Self(Rc::new(move || AnyView::new(f())))
    }

    #[must_use] 
    pub fn call(&self) -> AnyView {
        (self.0)()
    }
}

impl Default for ViewFn {
    fn default() -> Self {
        Self(Rc::new(|| AnyView::new(())))
    }
}

impl View for ViewFn {
    type State = AnyViewState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        self.call().build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RebuildContext) {
        self.call().rebuild(state, ctx);
    }
}

impl View for Children {
    type State = AnyViewState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        self().build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RebuildContext) {
        self().rebuild(state, ctx);
    }
}

impl View for ChildrenFn {
    type State = AnyViewState;

    fn build(self, ctx: &mut BuildContext) -> Self::State {
        self().build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RebuildContext) {
        self().rebuild(state, ctx);
    }
}
