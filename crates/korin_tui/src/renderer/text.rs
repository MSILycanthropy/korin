use korin_style::Style;

use crate::{Buffer, Cell, buffer::BufferView};

pub fn render(buffer: &mut Buffer, view: &BufferView, text: &str, style: &Style) {
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
