use crate::{Render, render::RenderContext};
use std::any::{Any, TypeId};
use std::fmt::Debug;

pub trait IntoAny {
    fn into_any(self) -> AnyView;
}

pub struct AnyView {
    type_id: TypeId,
    value: Box<dyn Any>,

    build_fn: fn(Box<dyn Any>, &mut dyn RenderContext) -> AnyState,
    rebuild_fn: fn(Box<dyn Any>, &mut AnyState, &mut dyn RenderContext),
}

impl AnyView {
    #[doc(hidden)]
    pub const fn as_type_id(&self) -> TypeId {
        self.type_id
    }
}

impl Render for AnyView {
    type State = AnyState;

    fn build(self, ctx: &mut impl RenderContext) -> AnyState {
        (self.build_fn)(self.value, ctx)
    }

    fn rebuild(self, state: &mut AnyState, ctx: &mut impl RenderContext) {
        (self.rebuild_fn)(self.value, state, ctx);
    }
}

impl Debug for AnyView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyView")
            .field("type_id", &self.type_id)
            .finish_non_exhaustive()
    }
}

pub struct AnyState {
    inner: Box<dyn Any>,
}

impl AnyState {
    pub fn new<T: 'static>(state: T) -> Self {
        Self {
            inner: Box::new(state),
        }
    }

    pub fn downcast<T: 'static>(&self) -> Option<&T> {
        self.inner.downcast_ref()
    }

    pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.downcast_mut()
    }
}
