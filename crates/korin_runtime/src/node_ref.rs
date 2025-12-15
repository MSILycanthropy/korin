use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use korin_layout::Rect;
use korin_reactive::{Ref, use_context};

use crate::{NodeId, inner::RuntimeInner};

#[derive(Clone)]
pub struct NodeRef {
    inner: Ref<NodeId>,
}

impl NodeRef {
    #[must_use]
    pub fn new() -> Self {
        Self { inner: Ref::new() }
    }

    #[must_use]
    pub fn get(&self) -> Option<NodeId> {
        self.inner.get()
    }

    pub fn set(&self, id: NodeId) {
        self.inner.set(id);
    }

    pub fn focus(&self) {
        let Some(id) = self.get() else { return };

        with_runtime_mut(|runtime| runtime.focus_node(id));
    }

    #[must_use]
    pub fn rect(&self) -> Option<Rect> {
        let id = self.get()?;

        with_runtime(|runtime| runtime.layout.absolute_rect(id))?
    }
}

fn with_runtime<T>(f: impl FnOnce(&RuntimeInner) -> T) -> Option<T> {
    let runtime = use_context::<Arc<RwLock<RuntimeInner>>>()?;
    let guard = runtime.read().expect("poisoned");
    Some(f(&guard))
}

fn with_runtime_mut<T>(f: impl FnOnce(&mut RuntimeInner) -> T) -> Option<T> {
    let runtime = use_context::<Arc<RwLock<RuntimeInner>>>()?;
    let mut guard = runtime.write().expect("poisoned");
    Some(f(&mut guard))
}

impl Default for NodeRef {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for NodeRef {
    type Target = Ref<NodeId>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<NodeRef> for Ref<NodeId> {
    fn from(r: NodeRef) -> Self {
        r.inner
    }
}

impl From<&NodeRef> for Ref<NodeId> {
    fn from(r: &NodeRef) -> Self {
        r.inner.clone()
    }
}
