use bitflags::bitflags;

pub type EventHandler = Box<dyn Fn(&Event) + Send + Sync>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: Modifiers,
}

impl KeyEvent {
    #[must_use]
    pub const fn new(code: KeyCode, modifiers: Modifiers) -> Self {
        Self { code, modifiers }
    }

    #[must_use]
    pub const fn is_char(&self, c: char) -> bool {
        matches!(self.code, KeyCode::Char(ch) if ch == c)
    }

    #[must_use]
    pub const fn ctrl(&self) -> bool {
        self.modifiers.contains(Modifiers::CTRL)
    }

    #[must_use]
    pub const fn alt(&self) -> bool {
        self.modifiers.contains(Modifiers::ALT)
    }

    #[must_use]
    pub const fn shift(&self) -> bool {
        self.modifiers.contains(Modifiers::SHIFT)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Insert,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Esc,
    CapsLock,
    NumLock,
    ScrollLock,
    Pause,
    F(u8),
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Modifiers: u8 {
        const NONE  = 0b0000;
        const CTRL  = 0b0001;
        const ALT   = 0b0010;
        const SHIFT = 0b0100;
    }
}

#[cfg(feature = "crossterm")]
mod crossterm_impl {
    use crate::{Event, KeyCode, KeyEvent, Modifiers};
    use crossterm::event::{
        Event as CtEvent, KeyCode as CtKey, KeyEvent as CtKeyEvent, KeyModifiers,
    };

    impl Event {
        #[must_use]
        pub fn from_crossterm(value: CtEvent) -> Option<Self> {
            match value {
                CtEvent::Key(key) => Some(Self::Key(key.into())),
                CtEvent::Resize(width, height) => Some(Self::Resize(width, height)),
                _ => None,
            }
        }
    }

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

    impl From<CtKeyEvent> for KeyEvent {
        fn from(value: CtKeyEvent) -> Self {
            Self::new(value.code.into(), value.modifiers.into())
        }
    }
}
