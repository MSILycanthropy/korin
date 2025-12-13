use crate::Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resize<T: Send + Sync> {
    pub width: T,
    pub height: T,
}

impl<T: Send + Sync + 'static> Event for Resize<T> {
    fn bubbles() -> bool {
        false
    }
}

impl<T: Send + Sync> From<(T, T)> for Resize<T> {
    fn from(value: (T, T)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}
