use crate::events::Event;

slotmap::new_key_type! {
    pub struct HandlerId;
}

type EventCallback = dyn FnMut(&mut Event) + 'static;

/// An event handler that can be invoked during event dispatch.
pub struct EventHandler {
    callback: Box<EventCallback>,
}

impl EventHandler {
    pub fn new<F>(callback: F) -> Self
    where
        F: FnMut(&mut Event) + 'static,
    {
        Self {
            callback: Box::new(callback),
        }
    }

    pub fn call(&mut self, event: &mut Event) {
        (self.callback)(event);
    }
}

impl<F> From<F> for EventHandler
where
    F: FnMut(&mut Event) + 'static,
{
    fn from(callback: F) -> Self {
        Self::new(callback)
    }
}

impl std::fmt::Debug for EventHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventHandler")
            .field("callback", &"<fn>")
            .finish()
    }
}
