use std::any::{Any, TypeId};
use std::fmt::Debug;

use korin_tree::NodeId;

use crate::{Render, RenderContext};

pub struct AnyState {
    type_id: TypeId,
    root: Option<NodeId>,
    inner: Box<dyn Any + Send + Sync>,
}

impl AnyState {
    pub fn new<T: Send + Sync + 'static>(inner: T, type_id: TypeId, root: Option<NodeId>) -> Self {
        Self {
            type_id,
            root,
            inner: Box::new(inner),
        }
    }

    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        self.inner.downcast_ref()
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.downcast_mut()
    }

    pub const fn root(&self) -> Option<NodeId> {
        self.root
    }

    pub fn replace<T: Send + Sync + 'static>(
        &mut self,
        inner: T,
        type_id: TypeId,
        root: Option<NodeId>,
    ) {
        self.type_id = type_id;
        self.root = root;
        self.inner = Box::new(inner);
    }
}

impl Debug for AnyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyState")
            .field("type_id", &self.type_id)
            .finish_non_exhaustive()
    }
}

pub struct AnyView<Ctx> {
    type_id: TypeId,
    inner: Box<dyn ErasedView<Ctx>>,
}

impl<Ctx> AnyView<Ctx> {
    #[must_use]
    pub const fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn build(self, ctx: &mut Ctx) -> AnyState {
        self.inner.build(ctx)
    }

    pub fn rebuild(self, state: &mut AnyState, ctx: &mut Ctx) {
        self.inner.rebuild(state, ctx);
    }
}

impl<Ctx: RenderContext + Clone> Render<Ctx> for AnyView<Ctx> {
    type State = AnyState;

    fn build(self, ctx: &mut Ctx) -> Self::State {
        self.inner.build(ctx)
    }

    fn rebuild(self, state: &mut Self::State, ctx: &mut Ctx) {
        self.inner.rebuild(state, ctx);
    }
}

impl<Ctx> Debug for AnyView<Ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyView")
            .field("type_id", &self.type_id)
            .finish_non_exhaustive()
    }
}

trait ErasedView<Ctx>: Send + Sync {
    fn build(self: Box<Self>, ctx: &mut Ctx) -> AnyState;
    fn rebuild(self: Box<Self>, state: &mut AnyState, ctx: &mut Ctx);
}

pub struct View<T> {
    inner: T,
}

impl<T, Ctx> ErasedView<Ctx> for View<T>
where
    T: Render<Ctx> + Send + Sync + 'static,
    Ctx: RenderContext + Clone,
    T::State: Send + Sync + 'static,
{
    fn build(self: Box<Self>, ctx: &mut Ctx) -> AnyState {
        let type_id = TypeId::of::<T::State>();
        let root_before = ctx.last_created_node();
        let state = self.inner.build(ctx);
        let root_after = ctx.last_created_node();

        let root = if root_before == root_after {
            None
        } else {
            root_after
        };

        AnyState::new(state, type_id, root)
    }

    fn rebuild(self: Box<Self>, state: &mut AnyState, ctx: &mut Ctx) {
        let new_type_id = TypeId::of::<T::State>();
        let old_type_id = state.type_id;

        if old_type_id == new_type_id
            && let Some(s) = state.downcast_mut::<T::State>()
        {
            self.inner.rebuild(s, ctx);
            return;
        }

        if let Some(old_root) = state.root() {
            ctx.remove_node(old_root);
        }

        let root_before = ctx.last_created_node();
        let new_state = self.inner.build(ctx);
        let root_after = ctx.last_created_node();

        let root = if root_before == root_after {
            None
        } else {
            root_after
        };

        state.replace(new_state, new_type_id, root);
    }
}

pub trait IntoView<Ctx> {
    fn into_view(self) -> AnyView<Ctx>;
}

impl<T, Ctx> IntoView<Ctx> for T
where
    T: Render<Ctx> + Send + Sync + 'static,
    Ctx: RenderContext + Clone,
    T::State: Send + Sync + 'static,
{
    fn into_view(self) -> AnyView<Ctx> {
        AnyView {
            type_id: TypeId::of::<T>(),
            inner: Box::new(View { inner: self }),
        }
    }
}
