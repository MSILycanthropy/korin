use std::{ops::Deref, sync::Arc};

use crate::{Event, EventContext};

type HandlerInner<E> = Option<Arc<dyn Fn(&EventContext<E>) + Send + Sync>>;

#[derive(Clone)]
pub struct Handler<E: Event>(HandlerInner<E>);

impl<E: Event> Handler<E> {
    pub fn call(&self, ctx: &EventContext<E>) {
        if let Some(ref f) = self.0 {
            f(ctx);
        }
    }

    pub fn new(f: impl Fn(&EventContext<E>) + Send + Sync + 'static) -> Self {
        Self(Some(Arc::new(f)))
    }

    #[must_use]
    pub const fn none() -> Self {
        Self(None)
    }
}

impl<E: Event> Deref for Handler<E> {
    type Target = HandlerInner<E>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait IntoHandler<E: Event> {
    fn into_handler(self) -> Handler<E>;
}

impl<E: Event, F> IntoHandler<E> for F
where
    F: Fn(&EventContext<E>) + Send + Sync + 'static,
{
    fn into_handler(self) -> Handler<E> {
        Handler::new(self)
    }
}

impl<E: Event> IntoHandler<E> for Handler<E> {
    fn into_handler(self) -> Self {
        self
    }
}

impl<E: Event> IntoHandler<E> for Option<Handler<E>> {
    fn into_handler(self) -> Handler<E> {
        self.unwrap_or_else(Handler::none)
    }
}
