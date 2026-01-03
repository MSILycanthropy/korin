use capsule_corp::{
    BasicColor, BorderStyle, CapsuleDocument, CapsuleNode, Color, ComputedStyle, Display, Edges,
    FontStyle, FontWeight, TextDecoration,
};
use indextree::NodeId;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color as RatColor, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::Document;

pub fn paint(document: &Document, frame: &mut Frame) {
    let root = document.root;

    for child in document.children(root) {
        paint_node(document, child, frame, 0, 0);
    }
}

fn paint_node(document: &Document, id: NodeId, frame: &mut Frame, offset_x: u16, offset_y: u16) {
    let node = document.get_node(id);

    let layout = node.layout;

    let x = offset_x.saturating_add(layout.location.x);
    let y = offset_y.saturating_add(layout.location.y);

    let rect = Rect::new(
        x,
        y,
        layout.resolved_box.border_box_size().width,
        layout.resolved_box.border_box_size().height,
    );

    if let Some(text) = node.text_content() {
        let style = document
            .parent(id)
            .and_then(|node| document.get_node(node).computed_style())
            .map(convert_text_style)
            .unwrap_or_default();

        let paragraph = Paragraph::new(text).style(style);
        frame.render_widget(paragraph, rect);
        return;
    }

    let Some(style) = node.computed_style() else {
        return;
    };

    if matches!(style.display, Display::None) {
        return;
    }

    let borders = convert_borders(style.border_style);
    let mut block = Block::default()
        .style(Style::default().bg(convert_color(style.background_color)))
        .borders(borders);

    if !borders.is_empty() {
        block = block.border_style(Style::default().fg(convert_color(style.border_color.top)));
    }

    frame.render_widget(block, rect);

    let resolved = &layout.resolved_box;
    let content_x = x
        .saturating_add(resolved.border.left)
        .saturating_add(resolved.padding.left);
    let content_y = y
        .saturating_add(resolved.border.top)
        .saturating_add(resolved.padding.top);

    for child in document.children(id) {
        paint_node(document, child, frame, content_x, content_y);
    }
}

fn convert_text_style(style: &ComputedStyle) -> Style {
    let mut result = Style::default().fg(convert_color(style.color));

    if matches!(style.font_weight, FontWeight::Bold) {
        result = result.add_modifier(Modifier::BOLD);
    }

    if matches!(style.font_style, FontStyle::Italic) {
        result = result.add_modifier(Modifier::ITALIC);
    }

    match style.text_decoration {
        TextDecoration::Underline => result = result.add_modifier(Modifier::UNDERLINED),
        TextDecoration::Strikethrough => result = result.add_modifier(Modifier::CROSSED_OUT),
        TextDecoration::None => {}
    }

    result
}

const fn convert_color(color: Color) -> RatColor {
    match color {
        Color::Reset => RatColor::Reset,
        Color::Basic(basic) => convert_basic_color(basic),
        Color::Bright(basic) => convert_bright_color(basic),
        Color::Ansi(n) => RatColor::Indexed(n),
        Color::Rgb(r, g, b) => RatColor::Rgb(r, g, b),
    }
}

const fn convert_basic_color(color: BasicColor) -> RatColor {
    match color {
        BasicColor::Black => RatColor::Black,
        BasicColor::Red => RatColor::Red,
        BasicColor::Green => RatColor::Green,
        BasicColor::Yellow => RatColor::Yellow,
        BasicColor::Blue => RatColor::Blue,
        BasicColor::Magenta => RatColor::Magenta,
        BasicColor::Cyan => RatColor::Cyan,
        BasicColor::White => RatColor::Gray,
    }
}

const fn convert_bright_color(color: BasicColor) -> RatColor {
    match color {
        BasicColor::Black => RatColor::DarkGray,
        BasicColor::Red => RatColor::LightRed,
        BasicColor::Green => RatColor::LightGreen,
        BasicColor::Yellow => RatColor::LightYellow,
        BasicColor::Blue => RatColor::LightBlue,
        BasicColor::Magenta => RatColor::LightMagenta,
        BasicColor::Cyan => RatColor::LightCyan,
        BasicColor::White => RatColor::White,
    }
}

fn convert_borders(border_style: Edges<BorderStyle>) -> Borders {
    let mut borders = Borders::empty();

    if !matches!(border_style.top, BorderStyle::None) {
        borders |= Borders::TOP;
    }
    if !matches!(border_style.right, BorderStyle::None) {
        borders |= Borders::RIGHT;
    }
    if !matches!(border_style.bottom, BorderStyle::None) {
        borders |= Borders::BOTTOM;
    }
    if !matches!(border_style.left, BorderStyle::None) {
        borders |= Borders::LEFT;
    }

    borders
}
