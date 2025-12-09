mod context;
mod error;
mod inner;
mod node;

use std::{
    any::Any,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use inner::RuntimeInner;

pub use context::RuntimeContext;
pub use error::{RuntimeError, RuntimeResult};
use korin_layout::Size;
use korin_reactive::reactive_graph::owner::{Owner, provide_context};
use korin_view::{AnyView, Render};
pub use node::{Node, NodeContent};

pub type View = AnyView<RuntimeContext>;

pub trait IntoView: korin_view::IntoView<RuntimeContext> {
    fn into_view(self) -> View;
}

impl<T: korin_view::IntoView<RuntimeContext>> IntoView for T {
    fn into_view(self) -> View {
        korin_view::IntoView::into_view(self)
    }
}

pub struct Runtime {
    inner: Arc<RwLock<RuntimeInner>>,
    owner: Owner,
    state: Option<Box<dyn Any + Send + Sync>>,
}

impl Runtime {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RuntimeInner::new())),
            owner: Owner::new(),
            state: None,
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
        let state = view.build(&mut ctx);

        self.state = Some(Box::new(state));

        let mut inner = self.inner_mut();
        inner.update_focus_order();

        if let Some(first) = inner.focus.focused().or_else(|| {
            inner.focus.focus_next();
            inner.focus.focused()
        }) {
            let _ = inner.try_on_focus(first);
        }

        drop(inner);

        Ok(())
    }

    pub fn compute_layout(&mut self, size: Size) -> RuntimeResult<()> {
        self.inner_mut().compute_layout(size)
    }

    /// Returns the [`RuntimeInner`] of this [`Runtime`].
    ///
    /// # Panics
    ///
    /// Panics if the inner's `RwLock` is poisoned
    pub fn inner(&self) -> RwLockReadGuard<'_, RuntimeInner> {
        self.inner.read().expect("poisoned")
    }

    /// Returns a mutable version of [`RuntimeInner`] of this [`Runtime`].
    ///
    /// # Panics
    ///
    /// Panics if the inner's `RwLock` is poisoned
    pub fn inner_mut(&self) -> RwLockWriteGuard<'_, RuntimeInner> {
        self.inner.write().expect("poisoned")
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
