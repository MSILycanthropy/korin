mod context;
mod event;
mod events;
mod handler;
mod listeners;

pub use context::EventContext;
pub use event::Event;
pub use events::*;
pub use handler::*;
pub use listeners::Listeners;

#[cfg(feature = "crossterm")]
mod crossterm_impl {
    use crate::{Key, KeyCode, Modifiers, MouseButton, MouseDown, MouseMove, MouseUp, Scroll};
    use crossterm::event::{
        KeyCode as CtKey, KeyEvent, KeyModifiers, MouseButton as CtButton, MouseEvent,
        MouseEventKind,
    };
    use korin_geometry::Point;

    impl From<CtKey> for KeyCode {
        fn from(value: CtKey) -> Self {
            match value {
                CtKey::Char(c) => Self::Char(c),
                CtKey::Enter => Self::Enter,
                CtKey::Tab => Self::Tab,
                CtKey::BackTab => Self::BackTab,
                CtKey::Backspace => Self::Backspace,
                CtKey::Delete => Self::Delete,
                CtKey::Insert => Self::Insert,
                CtKey::Left => Self::Left,
                CtKey::Right => Self::Right,
                CtKey::Up => Self::Up,
                CtKey::Down => Self::Down,
                CtKey::Home => Self::Home,
                CtKey::End => Self::End,
                CtKey::PageUp => Self::PageUp,
                CtKey::PageDown => Self::PageDown,
                CtKey::Esc => Self::Esc,
                CtKey::CapsLock => Self::CapsLock,
                CtKey::NumLock => Self::NumLock,
                CtKey::ScrollLock => Self::ScrollLock,
                CtKey::Pause => Self::Pause,
                CtKey::F(n) => Self::F(n),
                _ => Self::Char('\0'),
            }
        }
    }

    impl From<KeyModifiers> for Modifiers {
        fn from(value: KeyModifiers) -> Self {
            let mut result = Self::NONE;
            if value.contains(KeyModifiers::CONTROL) {
                result |= Self::CTRL;
            }
            if value.contains(KeyModifiers::ALT) {
                result |= Self::ALT;
            }
            if value.contains(KeyModifiers::SHIFT) {
                result |= Self::SHIFT;
            }
            result
        }
    }

    impl From<KeyEvent> for Key {
        fn from(value: KeyEvent) -> Self {
            Self::new(value.code.into(), value.modifiers.into())
        }
    }

    impl From<CtButton> for MouseButton {
        fn from(value: CtButton) -> Self {
            match value {
                CtButton::Left => Self::Left,
                CtButton::Right => Self::Right,
                CtButton::Middle => Self::Middle,
            }
        }
    }

    impl TryFrom<MouseEvent> for MouseDown<u16> {
        type Error = ();

        fn try_from(event: MouseEvent) -> Result<Self, Self::Error> {
            match event.kind {
                MouseEventKind::Down(button) => Ok(Self {
                    position: Point::new(event.column, event.row),
                    button: button.into(),
                }),
                _ => Err(()),
            }
        }
    }

    impl TryFrom<MouseEvent> for MouseUp<u16> {
        type Error = ();

        fn try_from(event: MouseEvent) -> Result<Self, Self::Error> {
            match event.kind {
                MouseEventKind::Up(button) => Ok(Self {
                    position: Point::new(event.column, event.row),
                    button: button.into(),
                }),
                _ => Err(()),
            }
        }
    }

    impl TryFrom<MouseEvent> for MouseMove<u16> {
        type Error = ();

        fn try_from(event: MouseEvent) -> Result<Self, Self::Error> {
            match event.kind {
                MouseEventKind::Moved | MouseEventKind::Drag(_) => Ok(Self {
                    position: Point::new(event.column, event.row),
                }),
                _ => Err(()),
            }
        }
    }

    impl TryFrom<MouseEvent> for Scroll<u16, i16> {
        type Error = ();

        fn try_from(event: MouseEvent) -> Result<Self, Self::Error> {
            let (dx, dy) = match event.kind {
                MouseEventKind::ScrollUp => (0, -1),
                MouseEventKind::ScrollDown => (0, 1),
                MouseEventKind::ScrollLeft => (-1, 0),
                MouseEventKind::ScrollRight => (1, 0),
                _ => return Err(()),
            };
            Ok(Self {
                position: Point::new(event.column, event.row),
                delta: Point::new(dx, dy),
            })
        }
    }
}
