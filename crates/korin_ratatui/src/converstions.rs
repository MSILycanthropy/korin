use korin_layout::Rect;
use korin_style::{BorderStyle, Borders, Color, Modifiers, Style};
use ratatui::{
    layout::Rect as RatRect,
    style::{Color as RatColor, Modifier, Style as RatStyle},
    widgets::BorderType,
};

pub const fn to_rat_color(color: Color) -> RatColor {
    match color {
        Color::Reset => RatColor::Reset,
        Color::Black => RatColor::Black,
        Color::White => RatColor::White,
        Color::Red => RatColor::Red,
        Color::Green => RatColor::Green,
        Color::Blue => RatColor::Blue,
        Color::Yellow => RatColor::Yellow,
        Color::Magenta => RatColor::Magenta,
        Color::Cyan => RatColor::Cyan,
        Color::Gray => RatColor::Gray,
        Color::DarkGray => RatColor::DarkGray,
        Color::LightRed => RatColor::LightRed,
        Color::LightGreen => RatColor::LightGreen,
        Color::LightBlue => RatColor::LightBlue,
        Color::LightYellow => RatColor::LightYellow,
        Color::LightMagenta => RatColor::LightMagenta,
        Color::LightCyan => RatColor::LightCyan,
        Color::Rgb(r, g, b) => RatColor::Rgb(r, g, b),
        Color::Indexed(i) => RatColor::Indexed(i),
    }
}

pub const fn to_rat_border_type(style: BorderStyle) -> BorderType {
    match style {
        BorderStyle::Plain => BorderType::Plain,
        BorderStyle::Rounded => BorderType::Rounded,
        BorderStyle::Double => BorderType::Double,
        BorderStyle::Thick => BorderType::Thick,
    }
}

pub fn to_rat_modifier(mods: Modifiers) -> Modifier {
    let mut result = Modifier::empty();
    if mods.contains(Modifiers::BOLD) {
        result |= Modifier::BOLD;
    }
    if mods.contains(Modifiers::DIM) {
        result |= Modifier::DIM;
    }
    if mods.contains(Modifiers::ITALIC) {
        result |= Modifier::ITALIC;
    }
    if mods.contains(Modifiers::UNDERLINE) {
        result |= Modifier::UNDERLINED;
    }
    result
}

pub fn to_rat_style_text(style: &Style) -> RatStyle {
    RatStyle::default()
        .fg(to_rat_color(style.text_color))
        .bg(to_rat_color(style.background))
        .add_modifier(to_rat_modifier(style.text_modifiers))
}

pub fn to_rat_borders(borders: Borders) -> ratatui::widgets::Borders {
    let mut result = ratatui::widgets::Borders::empty();
    if borders.contains(Borders::TOP) {
        result |= ratatui::widgets::Borders::TOP;
    }
    if borders.contains(Borders::RIGHT) {
        result |= ratatui::widgets::Borders::RIGHT;
    }
    if borders.contains(Borders::BOTTOM) {
        result |= ratatui::widgets::Borders::BOTTOM;
    }
    if borders.contains(Borders::LEFT) {
        result |= ratatui::widgets::Borders::LEFT;
    }
    result
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub const fn to_rat_rect(rect: Rect) -> RatRect {
    RatRect {
        x: rect.x.max(0.0) as u16,
        y: rect.y.max(0.0) as u16,
        width: rect.width.max(0.0) as u16,
        height: rect.height.max(0.0) as u16,
    }
}
