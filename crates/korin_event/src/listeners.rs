use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{Event, EventContext, Handler, IntoHandler};

type ErasedHandler = Box<dyn Any + Send + Sync>;

pub struct Listeners {
    handlers: HashMap<TypeId, Vec<ErasedHandler>>,
}

impl Listeners {
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn add<E: Event>(&mut self, handler: impl IntoHandler<E>) {
        let handler = handler.into_handler();

        if handler.is_none() {
            return;
        }

        let type_id = TypeId::of::<E>();
        self.handlers
            .entry(type_id)
            .or_default()
            .push(Box::new(handler));
    }

    pub fn emit<E: Event>(&self, event: &E) -> bool {
        let type_id = TypeId::of::<E>();
        let context = EventContext::new(event);

        let Some(handlers) = self.handlers.get(&type_id) else {
            return false;
        };

        for handler in handlers {
            if let Some(h) = handler.downcast_ref::<Handler<E>>() {
                h.call(&context);

                if context.is_stopped() {
                    break;
                }
            }
        }

        context.is_stopped()
    }

    #[must_use]
    pub fn has<E: Event>(&self) -> bool {
        let type_id = TypeId::of::<E>();

        self.handlers.contains_key(&type_id)
    }
}

impl Default for Listeners {
    fn default() -> Self {
        Self::new()
    }
}
