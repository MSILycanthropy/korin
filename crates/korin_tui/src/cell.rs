use korin_style::{Color, Modifiers};
use std::fmt;
use unicode_width::UnicodeWidthChar;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub symbol: char,
    pub foreground: Color,
    pub background: Color,
    pub modifiers: Modifiers,
    pub skip: bool,
}

impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if *self == Self::EMPTY {
            return write!(f, "Cell::EMPTY");
        }

        let mut s = f.debug_struct("Cell");
        s.field("symbol", &self.symbol);

        if self.foreground != Color::Reset {
            s.field("fg", &self.foreground);
        }
        if self.background != Color::Reset {
            s.field("bg", &self.background);
        }
        if !self.modifiers.is_empty() {
            s.field("modifiers", &self.modifiers);
        }
        if self.skip {
            s.field("skip", &true);
        }

        s.finish()
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

impl Cell {
    pub const EMPTY: Self = Self::new(' ');

    #[must_use]
    pub const fn new(symbol: char) -> Self {
        Self {
            symbol,
            foreground: Color::Reset,
            background: Color::Reset,
            modifiers: Modifiers::NONE,
            skip: false,
        }
    }

    #[must_use]
    pub fn width(&self) -> usize {
        self.symbol.width().unwrap_or(0)
    }

    #[must_use]
    pub const fn foreground(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }

    #[must_use]
    pub const fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    #[must_use]
    pub const fn modifiers(mut self, modifiers: Modifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    #[must_use]
    pub const fn skip(mut self) -> Self {
        self.skip = true;
        self
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::EMPTY
    }
}
