use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Ref<T> {
    inner: Arc<RwLock<Option<T>>>,
}

impl<T> Ref<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.with_inner(Clone::clone)
    }

    pub fn set(&self, value: T) {
        self.with_inner_mut(|v| *v = Some(value));
    }

    pub fn clear(&self) {
        self.with_inner_mut(|v| *v = None);
    }

    fn with_inner<R>(&self, f: impl FnOnce(&Option<T>) -> R) -> R {
        f(&self.inner.read().expect("poisoned"))
    }

    fn with_inner_mut<R>(&self, f: impl FnOnce(&mut Option<T>) -> R) -> R {
        f(&mut self.inner.write().expect("poisoned"))
    }
}

impl<T: Send + Sync + Clone + 'static> Default for Ref<T> {
    fn default() -> Self {
        Self::new()
    }
}
