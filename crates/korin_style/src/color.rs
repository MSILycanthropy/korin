#[derive(Default, Clone, Copy, PartialEq, Eq)]
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
