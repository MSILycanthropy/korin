use korin_style::{BorderStyle, Borders, Style};

use crate::{Buffer, Cell, buffer::BufferView, symbols::BorderSymbols};

pub fn render(buffer: &mut Buffer, view: &BufferView, style: &Style) {
    view.fill(buffer, style);

    render_borders(buffer, view, style);
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
