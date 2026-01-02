pub use keyboard_types::{Code, Key, Location, Modifiers, NamedKey};

/// Keyboard event data.
///
/// Specification: <https://w3c.github.io/uievents/#interface-keyboardevent>
#[derive(Clone, Debug)]
pub struct KeyboardEvent {
    pub key: Key,
    pub code: Code,
    pub modifiers: Modifiers,
    pub repeat: bool,
    pub is_composing: bool,
    pub location: Location,
}
