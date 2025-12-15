use std::sync::Arc;

pub struct LazyFn<T>(Arc<dyn Fn() -> T + Send + Sync>);

impl<T> LazyFn<T> {
    pub fn new(f: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Self(Arc::new(f))
    }

    #[must_use]
    pub fn call(&self) -> T {
        let f = &self.0;

        f()
    }
}

impl<T> Clone for LazyFn<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub trait IntoLazyFn<T> {
    fn into_lazy(self) -> LazyFn<T>;
}

impl<T, F> IntoLazyFn<T> for F
where
    F: Fn() -> T + Send + Sync + 'static,
{
    fn into_lazy(self) -> LazyFn<T> {
        LazyFn::new(self)
    }
}

impl<T> IntoLazyFn<T> for LazyFn<T> {
    fn into_lazy(self) -> Self {
        self
    }
}
