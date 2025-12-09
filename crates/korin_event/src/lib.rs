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
