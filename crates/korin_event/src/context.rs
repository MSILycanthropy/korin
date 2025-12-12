use std::{cell::Cell, ops::Deref};

use crate::Event;

pub struct EventContext<'a, E: Event> {
    event: &'a E,
    stopped: Cell<bool>,
}

impl<'a, E: Event> EventContext<'a, E> {
    pub const fn new(event: &'a E) -> Self {
        Self {
            event,
            stopped: Cell::new(false),
        }
    }

    pub fn stop_propagation(&self) {
        self.stopped.set(true);
    }

    pub const fn is_stopped(&self) -> bool {
        self.stopped.get()
    }
}

impl<E: Event> Deref for EventContext<'_, E> {
    type Target = E;
    
    fn deref(&self) -> &Self::Target {
        self.event
    }
}
