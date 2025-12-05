use ratatui_core::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::{border, merge::MergeStrategy},
};

use crate::border::Borders;

// https://github.com/ratatui/ratatui/blob/main/ratatui-widgets/src/block.rs
pub trait Bordered {
    fn borders(&self) -> Borders;
    fn border_merging(&self) -> MergeStrategy;
    fn border_set(&self) -> border::Set<'_>;
    fn border_style(&self) -> Style;

    fn render_borders(&self, area: Rect, buf: &mut Buffer) {
        self.render_border_sides(area, buf);
        self.render_border_corners(area, buf);
    }

    fn render_border_sides(&self, area: Rect, buf: &mut Buffer) {
        let left = area.left();
        let top = area.top();
        // area.right() and area.bottom() are outside the rect, subtract 1 to get the last row/col
        let right = area.right() - 1;
        let bottom = area.bottom() - 1;

        // The first and last element of each line are not drawn when there is an adjacent line as
        // this would cause the corner to initially be merged with a side character and then a
        // corner character to be drawn on top of it. Some merge strategies would not produce a
        // correct character in that case.
        let is_replace = self.border_merging() != MergeStrategy::Replace;
        let left_inset = left + u16::from(is_replace && self.borders().contains(Borders::LEFT));
        let top_inset = top + u16::from(is_replace && self.borders().contains(Borders::TOP));
        let right_inset = right - u16::from(is_replace && self.borders().contains(Borders::RIGHT));
        let bottom_inset =
            bottom - u16::from(is_replace && self.borders().contains(Borders::BOTTOM));

        let sides = [
            (
                Borders::LEFT,
                left..=left,
                top_inset..=bottom_inset,
                self.border_set().vertical_left,
            ),
            (
                Borders::TOP,
                left_inset..=right_inset,
                top..=top,
                self.border_set().horizontal_top,
            ),
            (
                Borders::RIGHT,
                right..=right,
                top_inset..=bottom_inset,
                self.border_set().vertical_right,
            ),
            (
                Borders::BOTTOM,
                left_inset..=right_inset,
                bottom..=bottom,
                self.border_set().horizontal_bottom,
            ),
        ];
        for (border, x_range, y_range, symbol) in sides {
            if self.borders().contains(border) {
                for x in x_range {
                    for y in y_range.clone() {
                        buf[(x, y)]
                            .merge_symbol(symbol, self.border_merging())
                            .set_style(self.border_style());
                    }
                }
            }
        }
    }

    fn render_border_corners(&self, area: Rect, buf: &mut Buffer) {
        let corners = [
            (
                Borders::RIGHT | Borders::BOTTOM,
                area.right() - 1,
                area.bottom() - 1,
                self.border_set().bottom_right,
            ),
            (
                Borders::RIGHT | Borders::TOP,
                area.right() - 1,
                area.top(),
                self.border_set().top_right,
            ),
            (
                Borders::LEFT | Borders::BOTTOM,
                area.left(),
                area.bottom() - 1,
                self.border_set().bottom_left,
            ),
            (
                Borders::LEFT | Borders::TOP,
                area.left(),
                area.top(),
                self.border_set().top_left,
            ),
        ];

        for (border, x, y, symbol) in corners {
            if self.borders().contains(border) {
                buf[(x, y)]
                    .merge_symbol(symbol, self.border_merging())
                    .set_style(self.border_style());
            }
        }
    }
}
