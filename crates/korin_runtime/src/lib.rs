mod context;
mod error;
mod inner;
mod node;

use std::sync::{Arc, RwLock, RwLockWriteGuard};

use inner::RuntimeInner;

pub use context::RuntimeContext;
pub use error::{RuntimeError, RuntimeResult};
use korin_reactive::reactive_graph::owner::{Owner, provide_context};
use korin_view::Render;
pub use node::{Node, NodeContent};

#[derive(Clone)]
pub struct Runtime {
    inner: Arc<RwLock<RuntimeInner>>,
    owner: Owner,
}

impl Runtime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RuntimeInner::new())),
            owner: Owner::new(),
        }
    }

    pub fn mount<V>(&mut self, view: V) -> RuntimeResult<()>
    where
        V: Render<RuntimeContext>,
        V::State: 'static,
    {
        self.owner.set();

        provide_context(self.inner.clone());

        let mut ctx = RuntimeContext::new(self.inner.clone());
        let _state = view.build(&mut ctx);

        self.inner_mut().update_focus_order();

        Ok(())
    }

    fn inner_mut(&self) -> RwLockWriteGuard<'_, RuntimeInner> {
        self.inner.write().expect("poisoned")
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
