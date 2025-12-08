use std::any::{Any, TypeId};
use std::fmt::Debug;

use crate::{Render, RenderContext};

pub struct AnyState {
    type_id: TypeId,
    inner: Box<dyn Any + Send + Sync>,
}

impl AnyState {
    pub fn new<T: Send + Sync + 'static>(inner: T, type_id: TypeId) -> Self {
        Self {
            type_id,
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
        let state = self.inner.build(ctx);

        AnyState::new(state, type_id)
    }

    fn rebuild(self: Box<Self>, state: &mut AnyState, ctx: &mut Ctx) {
        if let Some(s) = state.downcast_mut::<T::State>() {
            self.inner.rebuild(s, ctx);
        }
    }
}

pub trait IntoAny<Ctx> {
    fn into_any(self) -> AnyView<Ctx>;
}

impl<T, Ctx> IntoAny<Ctx> for T
where
    T: Render<Ctx> + Send + Sync + 'static,
    Ctx: RenderContext + Clone,
    T::State: Send + Sync + 'static,
{
    fn into_any(self) -> AnyView<Ctx> {
        AnyView {
            type_id: TypeId::of::<T>(),
            inner: Box::new(View { inner: self }),
        }
    }
}
