use std::any::Any;

use crate::{Render, render::RenderContext};

pub trait IntoAny {
    fn into_any(self) -> AnyView;
}

pub struct AnyView {
    build_fn: Box<dyn FnOnce(&mut RenderContext) -> AnyState>,
}

impl AnyView {
    pub fn new<V>(view: V) -> Self
    where
        V: Render + 'static,
        V::State: 'static,
    {
        Self {
            build_fn: Box::new(move |ctx| {
                let state = view.build(ctx);
                AnyState::new(state)
            }),
        }
    }

    pub fn build(self, ctx: &mut RenderContext) -> AnyState {
        (self.build_fn)(ctx)
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
