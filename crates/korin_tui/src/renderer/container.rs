use korin_runtime::Node;
use korin_style::{BorderStyle, Borders, Overflow, Style};

use crate::{
    Buffer, Cell,
    buffer::BufferView,
    symbols::{self, BorderSymbols},
};

pub fn render(buffer: &mut Buffer, view: &BufferView, node: &Node) {
    let style = &node.computed_style;
    view.fill(buffer, style);

    render_borders(buffer, view, style);

    if style.overflow_y() == Overflow::Scroll {
        render_vertical_scrollbar(buffer, view, node);
    }

    if style.overflow_x() == Overflow::Scroll {
        render_horizontal_scrollbar(buffer, view, node);
    }
}

fn render_borders(buffer: &mut Buffer, view: &BufferView, style: &Style) {
    if style.borders().is_empty() {
        return;
    }

    if view.is_empty() {
        return;
    }

    let symbols = match style.border_style() {
        BorderStyle::Plain => &BorderSymbols::PLAIN,
        BorderStyle::Rounded => &BorderSymbols::ROUNDED,
        BorderStyle::Double => &BorderSymbols::DOUBLE,
        BorderStyle::Thick => &BorderSymbols::THICK,
    };

    let cell = |ch| {
        Cell::new(ch)
            .foreground(style.border_color())
            .background(style.background())
    };

    let x1 = view.width() - 1;
    let y1 = view.height() - 1;
    let borders = style.borders();

    if borders.contains(Borders::TOP) {
        for x in 1..x1 {
            view.set(buffer, x, 0, cell(symbols.h));
        }
    }

    if borders.contains(Borders::BOTTOM) {
        for x in 1..x1 {
            view.set(buffer, x, y1, cell(symbols.h));
        }
    }

    if borders.contains(Borders::LEFT) {
        for y in 1..y1 {
            view.set(buffer, 0, y, cell(symbols.v));
        }
    }

    if borders.contains(Borders::RIGHT) {
        for y in 1..y1 {
            view.set(buffer, x1, y, cell(symbols.v));
        }
    }

    if borders.contains(Borders::TOP | Borders::LEFT) {
        view.set(buffer, 0, 0, cell(symbols.tl));
    }

    if borders.contains(Borders::TOP | Borders::RIGHT) {
        view.set(buffer, x1, 0, cell(symbols.tr));
    }

    if borders.contains(Borders::BOTTOM | Borders::LEFT) {
        view.set(buffer, 0, y1, cell(symbols.bl));
    }

    if borders.contains(Borders::BOTTOM | Borders::RIGHT) {
        view.set(buffer, x1, y1, cell(symbols.br));
    }
}

fn render_vertical_scrollbar(buffer: &mut Buffer, view: &BufferView, node: &Node) {
    let viewport_height = f32::from(view.height());
    let content_height = node.content_size.height;

    if content_height <= viewport_height {
        return;
    }

    let borders = node.style.borders();
    let top_offset = u16::from(borders.contains(Borders::TOP));
    let bottom_offset = u16::from(borders.contains(Borders::BOTTOM));
    let right_offset = u16::from(borders.contains(Borders::RIGHT));

    let track_height = f32::from(view.height().saturating_sub(top_offset + bottom_offset));
    let thumb_height = ((viewport_height / content_height) * track_height).max(1.0);
    let max_scroll = content_height - viewport_height;
    let thumb_offset = (node.scroll_offset.y / max_scroll) * (track_height - thumb_height);

    let x = view.width() - 1 - right_offset;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    for y in 0..track_height as u16 {
        let in_thumb = f32::from(y) >= thumb_offset && f32::from(y) < thumb_offset + thumb_height;
        let symbol = if in_thumb {
            symbols::VERTICAL_THUMB
        } else {
            symbols::VERTICAL_TRACK
        };

        view.set(buffer, x, y + 1, Cell::new(symbol));
    }
}

fn render_horizontal_scrollbar(buffer: &mut Buffer, view: &BufferView, node: &Node) {
    let viewport_width = f32::from(view.width());
    let content_width = node.content_size.width;

    if content_width <= viewport_width {
        return;
    }

    let borders = node.style.borders();
    let left_offset = u16::from(borders.contains(Borders::LEFT));
    let right_offset = u16::from(borders.contains(Borders::RIGHT));
    let bottom_offset = u16::from(borders.contains(Borders::BOTTOM));

    let track_width = f32::from(view.width().saturating_sub(left_offset + right_offset));
    let thumb_width = ((viewport_width / content_width) * track_width).max(1.0);
    let max_scroll = content_width - viewport_width;
    let thumb_offset = (node.scroll_offset.x / max_scroll) * (track_width - thumb_width);

    let y = view.height() - 1 - bottom_offset;

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    for x in 0..track_width as u16 {
        let in_thumb = f32::from(x) >= thumb_offset && f32::from(x) < thumb_offset + thumb_width;
        let symbol = if in_thumb {
            symbols::HORIZONTAL_THUMB
        } else {
            symbols::HORIZONTAL_TRACK
        };

        view.set(buffer, x + 1, y, Cell::new(symbol));
    }
}
