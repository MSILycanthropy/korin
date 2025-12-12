use std::ops::Deref;

use crate::{Event, KeyEvent};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key(pub KeyEvent);

impl Event for Key {}

impl Deref for Key {
    type Target = KeyEvent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Focus;

impl Event for Focus {
    fn bubbles() -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Blur;

impl Event for Blur {
    fn bubbles() -> bool {
        false
    }
}

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
