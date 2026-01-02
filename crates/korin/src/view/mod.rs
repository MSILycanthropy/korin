mod any_view;
mod children;
mod context;
mod either;
mod element;
mod for_loop;
mod fragment;
pub mod html_elements;
mod mountable;
mod show;
mod text;

pub use any_view::{AnyView, AnyViewState};
pub use children::{Children, ChildrenFn, ChildrenFnMut, ViewFn};
pub use context::{BuildContext, RebuildContext};
pub use either::{Either, EitherState};
pub use element::{ElementView, ElementViewState};
pub use for_loop::for_each;
pub use fragment::{Fragment, FragmentState};
pub use html_elements::*;
pub use mountable::Mountable;
pub use show::{show, show_if, show_unless};
pub use text::{TextView, TextViewState};
/// A View is a declarative description of UI that is built into DOM nodes
///
/// Views are consumed during `build()` to produce `State`, which holds
/// references to the created DOM nodes. The `rebuild()` method updates
/// dynamic content without recreating DOM node structure.
pub trait View {
    /// State produced by building this view, typically contains `NodeIds`
    type State: Mountable;

    /// Build the view into DOM nodes, returning state for future rebuilds
    /// DOM nodes are not attached until `mount` is called
    fn build(self, ctx: &mut BuildContext) -> Self::State;

    /// Update dynamic content using the existing state
    fn rebuild(self, state: &mut Self::State, ctx: &mut RebuildContext);
}

impl View for () {
    type State = ();

    fn build(self, _ctx: &mut BuildContext) -> Self::State {}

    fn rebuild(self, _state: &mut Self::State, _ctx: &mut RebuildContext) {}
}
