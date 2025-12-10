#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    #[default]
    Reset,
    Black,
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightBlue,
    LightYellow,
    LightMagenta,
    LightCyan,
    Rgb(u8, u8, u8),
    Indexed(u8),
}

#[cfg(feature = "crossterm")]
mod ct_impl {
    use crate::Color;
    use crossterm::style::Color as CtColor;

    impl From<Color> for CtColor {
        fn from(value: Color) -> Self {
            match value {
                Color::Reset => Self::Reset,
                Color::Black => Self::Black,
                Color::White => Self::White,
                Color::Red => Self::DarkRed,
                Color::Green => Self::DarkGreen,
                Color::Blue => Self::DarkBlue,
                Color::Yellow => Self::DarkYellow,
                Color::Magenta => Self::DarkMagenta,
                Color::Cyan => Self::DarkCyan,
                Color::Gray => Self::Grey,
                Color::DarkGray => Self::DarkGrey,
                Color::LightRed => Self::Red,
                Color::LightGreen => Self::Green,
                Color::LightBlue => Self::Blue,
                Color::LightYellow => Self::Yellow,
                Color::LightMagenta => Self::Magenta,
                Color::LightCyan => Self::Cyan,
                Color::Rgb(r, g, b) => Self::Rgb { r, g, b },
                Color::Indexed(i) => Self::AnsiValue(i),
            }
        }
    }
}
