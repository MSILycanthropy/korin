use korin_reactive::reactive_graph::owner::Owner;
use korin_tree::NodeId;
use korin_view::{AnyState, Render};

use crate::RuntimeContext;

#[derive(Clone)]
pub struct OwnedView<T> {
    owner: Owner,
    view: T,
}

impl<T> OwnedView<T> {
    /// Creates a new [`OwnedView<T>`].
    ///
    /// # Panics
    ///
    /// Panics if there is no [`Owner`].
    pub fn new(view: T) -> Self {
        let owner = Owner::current().expect("no reactive owner");
        Self { owner, view }
    }

    pub const fn new_with_owner(view: T, owner: Owner) -> Self {
        Self { owner, view }
    }
}

pub struct OwnedViewState {
    owner: Owner,
    state: AnyState,
}

impl OwnedViewState {
    const fn new(state: AnyState, owner: Owner) -> Self {
        Self { owner, state }
    }

    #[must_use]
    pub const fn root(&self) -> Option<NodeId> {
        self.state.root()
    }
}

impl<T> Render<RuntimeContext> for OwnedView<T>
where
    T: crate::IntoView,
{
    type State = OwnedViewState;

    fn build(self, ctx: &mut RuntimeContext) -> Self::State {
        let state = self.owner.with(|| {
            let view = crate::IntoView::into_view(self.view);
            view.build(ctx)
        });
        OwnedViewState::new(state, self.owner)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut RuntimeContext) {
        self.owner.with(|| {
            let view = crate::IntoView::into_view(self.view);
            view.rebuild(&mut state.state, ctx);
        });
        state.owner = self.owner;
    }
}
