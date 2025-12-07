use std::any::Any;

pub struct EventHandler {
    inner: Box<dyn Any>,
}

impl EventHandler {
    pub fn new<E: 'static>(handler: impl Fn(&E) + 'static) -> Self {
        let boxed: Box<dyn Fn(&E)> = Box::new(handler);
        Self {
            inner: Box::new(boxed),
        }
    }

    pub fn call<E: 'static>(&self, event: &E) {
        if let Some(handler) = self.inner.downcast_ref::<Box<dyn Fn(&E)>>() {
            handler(event);
        }
    }
}

pub type FocusHandler = Box<dyn Fn()>;
