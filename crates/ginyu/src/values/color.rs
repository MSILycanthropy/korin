#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Color {
    #[default]
    Reset,
    Basic(BasicColor),
    Bright(BasicColor),
    Ansi(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    pub const BLACK: Self = Self::Basic(BasicColor::Black);
    pub const RED: Self = Self::Basic(BasicColor::Red);
    pub const GREEN: Self = Self::Basic(BasicColor::Green);
    pub const YELLOW: Self = Self::Basic(BasicColor::Yellow);
    pub const BLUE: Self = Self::Basic(BasicColor::Blue);
    pub const MAGENTA: Self = Self::Basic(BasicColor::Magenta);
    pub const CYAN: Self = Self::Basic(BasicColor::Cyan);
    pub const WHITE: Self = Self::Basic(BasicColor::White);
}

/// Basic terminal colors (ANSI 0-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BasicColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
}

impl BasicColor {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "black" => Some(Self::Black),
            "red" => Some(Self::Red),
            "green" => Some(Self::Green),
            "yellow" => Some(Self::Yellow),
            "blue" => Some(Self::Blue),
            "magenta" => Some(Self::Magenta),
            "cyan" => Some(Self::Cyan),
            "white" => Some(Self::White),
            _ => None,
        }
    }

    #[must_use]
    pub const fn ansi_code(self) -> u8 {
        self as u8
    }

    #[must_use]
    pub const fn bright_ansi_code(self) -> u8 {
        self as u8 + 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_color_from_name() {
        assert_eq!(BasicColor::from_name("red"), Some(BasicColor::Red));
        assert_eq!(BasicColor::from_name("cyan"), Some(BasicColor::Cyan));
        assert_eq!(BasicColor::from_name("purple"), None);
    }

    #[test]
    fn basic_color_ansi_codes() {
        assert_eq!(BasicColor::Black.ansi_code(), 0);
        assert_eq!(BasicColor::White.ansi_code(), 7);
        assert_eq!(BasicColor::Red.bright_ansi_code(), 9);
    }

    #[test]
    fn color_default() {
        assert_eq!(Color::default(), Color::Reset);
    }

    #[test]
    fn color_constants() {
        assert_eq!(Color::RED, Color::Basic(BasicColor::Red));
        assert_eq!(Color::CYAN, Color::Basic(BasicColor::Cyan));
    }
}
