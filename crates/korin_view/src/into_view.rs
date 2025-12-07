use crate::Render;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View<T>
where
    T: Sized,
{
    inner: T,
}

impl<T> View<T> {
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

pub trait IntoView
where
    Self: Sized + Render + Send,
{
    fn into_view(self) -> View<Self>;
}

impl<T> IntoView for T
where
    T: Sized + Render + Send,
{
    fn into_view(self) -> View<Self> {
        View::new(self)
    }
}
