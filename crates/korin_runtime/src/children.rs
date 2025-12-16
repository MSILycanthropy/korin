use std::sync::Arc;

use crate::{IntoView, View};

/// Trait for converting closures into children types.
/// Used by the `view!` macro to pass children to components.
pub trait ToChildren<F> {
    fn to_children(f: F) -> Self;
}

/// The most common type for the `children` property on components.
/// Can only be called once, type-erased to `View`.
///
/// Use when children are rendered once and never need to re-render.
pub type Children = Box<dyn FnOnce() -> View + Send>;

impl<F, V> ToChildren<F> for Children
where
    F: FnOnce() -> V + Send + 'static,
    V: IntoView + 'static,
{
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_view())
    }
}

/// Like `Children`, but can be called multiple times.
/// Type-erased to `View`.
///
/// Use when children may need to re-render reactively (e.g., in `Show`).
pub type ChildrenFn = Arc<dyn Fn() -> View + Send + Sync>;

impl<F, V> ToChildren<F> for ChildrenFn
where
    F: Fn() -> V + Send + Sync + 'static,
    V: IntoView + 'static,
{
    fn to_children(f: F) -> Self {
        Arc::new(move || f().into_view())
    }
}

/// Returns a collection of children that can be iterated over.
/// Can only be called once.
///
/// Use for components like `Tabs` that need to process each child individually.
pub type ChildrenFragment = Box<dyn FnOnce() -> Vec<View> + Send>;

impl<F, V> ToChildren<F> for ChildrenFragment
where
    F: FnOnce() -> Vec<V> + Send + 'static,
    V: IntoView + 'static,
{
    fn to_children(f: F) -> Self {
        Box::new(move || f().into_iter().map(IntoView::into_view).collect())
    }
}

/// Like `ChildrenFragment`, but can be called multiple times.
///
/// Use for components that need to iterate children and may re-render.
pub type ChildrenFragmentFn = Arc<dyn Fn() -> Vec<View> + Send + Sync>;

impl<F, V> ToChildren<F> for ChildrenFragmentFn
where
    F: Fn() -> Vec<V> + Send + Sync + 'static,
    V: IntoView + 'static,
{
    fn to_children(f: F) -> Self {
        Arc::new(move || f().into_iter().map(IntoView::into_view).collect())
    }
}

/// Typed version of `Children`. Preserves the concrete type `T` instead of
/// erasing to `View`, allowing the compiler to optimize more effectively.
/// Can only be called once.
pub struct TypedChildren<T>(Box<dyn FnOnce() -> T + Send>);

impl<T> TypedChildren<T> {
    pub fn into_inner(self) -> impl FnOnce() -> T + Send {
        self.0
    }

    #[must_use]
    pub fn call(self) -> T {
        (self.0)()
    }
}

impl<F, T> ToChildren<F> for TypedChildren<T>
where
    F: FnOnce() -> T + Send + 'static,
{
    fn to_children(f: F) -> Self {
        Self(Box::new(f))
    }
}

/// Typed version of `ChildrenFn`. Preserves the concrete type `T` instead of
/// erasing to `View`, allowing the compiler to optimize more effectively.
/// Can be called multiple times.
pub struct TypedChildrenFn<T>(Arc<dyn Fn() -> T + Send + Sync>);

impl<T> std::fmt::Debug for TypedChildrenFn<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TypedChildrenFn").finish()
    }
}

impl<T> Clone for TypedChildrenFn<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> TypedChildrenFn<T> {
    #[must_use]
    pub fn into_inner(self) -> Arc<dyn Fn() -> T + Send + Sync> {
        self.0
    }

    #[must_use]
    pub fn call(&self) -> T {
        (self.0)()
    }
}

impl<F, T> ToChildren<F> for TypedChildrenFn<T>
where
    F: Fn() -> T + Send + Sync + 'static,
{
    fn to_children(f: F) -> Self {
        Self(Arc::new(f))
    }
}

/// A wrapper for optional view props like `fallback`.
/// Has a `Default` impl that returns an empty view, so components
/// can call `fallback.run()` without checking for `None`.
/// Can be called multiple times.
#[derive(Clone)]
pub struct ViewFn(Arc<dyn Fn() -> View + Send + Sync>);

impl Default for ViewFn {
    fn default() -> Self {
        Self(Arc::new(|| ().into_view()))
    }
}

impl ViewFn {
    #[must_use]
    pub fn run(&self) -> View {
        (self.0)()
    }
}

impl<F, V> From<F> for ViewFn
where
    F: Fn() -> V + Send + Sync + 'static,
    V: IntoView + 'static,
{
    fn from(f: F) -> Self {
        Self(Arc::new(move || f().into_view()))
    }
}

/// Like `ViewFn`, but can only be called once.
/// Has a `Default` impl that returns an empty view.
pub struct ViewFnOnce(Box<dyn FnOnce() -> View + Send>);

impl Default for ViewFnOnce {
    fn default() -> Self {
        Self(Box::new(|| ().into_view()))
    }
}

impl ViewFnOnce {
    #[must_use]
    pub fn run(self) -> View {
        (self.0)()
    }
}

impl<F, V> From<F> for ViewFnOnce
where
    F: FnOnce() -> V + Send + 'static,
    V: IntoView + 'static,
{
    fn from(f: F) -> Self {
        Self(Box::new(move || f().into_view()))
    }
}
