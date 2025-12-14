use korin_style::{Style, WhiteSpace};

use crate::{Buffer, Cell, buffer::BufferView};

pub fn render(buffer: &mut Buffer, view: &BufferView, text: &str, style: &Style) {
    let wrap = style.white_space() == WhiteSpace::Normal;

    if wrap {
        render_wrap(buffer, view, text, style);
        return;
    }

    render_nowrap(buffer, view, text, style);
}

fn render_wrap(buffer: &mut Buffer, view: &BufferView, text: &str, style: &Style) {
    let max_width = view.width() as usize;

    let mut y = 0u16;

    for line in text.lines() {
        let mut x = 0usize;

        for ch in line.chars() {
            if x >= max_width {
                y += 1;
                x = 0;
            }

            view.set(
                buffer,
                u16::try_from(x).unwrap_or(u16::MAX),
                y,
                Cell::new(ch)
                    .foreground(style.text_color())
                    .background(style.background())
                    .modifiers(style.text_modifiers()),
            );

            x += 1;
        }

        y += 1;
    }
}

fn render_nowrap(buffer: &mut Buffer, view: &BufferView, text: &str, style: &Style) {
    for (y, line) in text.lines().enumerate() {
        let y = u16::try_from(y).unwrap_or(u16::MAX);

        for (x, ch) in line.chars().enumerate() {
            let x = u16::try_from(x).unwrap_or(u16::MAX);

            view.set(
                buffer,
                x,
                y,
                Cell::new(ch)
                    .foreground(style.text_color())
                    .background(style.background())
                    .modifiers(style.text_modifiers()),
            );
        }
    }
}
