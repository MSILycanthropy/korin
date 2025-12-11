use korin_style::Style;

use crate::{Cell, Position, Rect, Size};

pub struct Buffer {
    cells: Vec<Cell>,
    size: Size,
}

impl Buffer {
    #[must_use]
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            cells: vec![Cell::EMPTY; (width as usize) * (height as usize)],
            size: Size::new(width, height),
        }
    }

    #[must_use]
    pub const fn view(&self) -> BufferView {
        BufferView {
            area: Rect::new(0, 0, self.size.width, self.size.height),
        }
    }

    #[must_use]
    pub const fn size(&self) -> Size {
        self.size
    }

    #[must_use]
    pub const fn cells(&self) -> &[Cell] {
        self.cells.as_slice()
    }

    pub fn resize(&mut self, size: Size) {
        self.size = size;
        self.cells
            .resize((size.width as usize) * (size.height as usize), Cell::EMPTY);
        self.clear();
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::EMPTY);
    }

    #[must_use]
    pub fn get(&self, position: Position) -> Option<&Cell> {
        if !self.within_bounds(position) {
            return None;
        }

        Some(&self.cells[self.index(position)])
    }

    pub fn get_mut(&mut self, position: Position) -> Option<&mut Cell> {
        if !self.within_bounds(position) {
            return None;
        }

        let index = self.index(position);
        Some(&mut self.cells[index])
    }

    pub fn set(&mut self, position: Position, cell: Cell) {
        if !self.within_bounds(position) {
            return;
        }

        let index = self.index(position);
        self.cells[index] = cell;
    }

    pub fn diff<'a>(&'a self, previous: &'a Self) -> impl Iterator<Item = (u16, u16, &'a Cell)> {
        let width = self.size.width as usize;

        self.cells
            .iter()
            .zip(previous.cells.iter())
            .enumerate()
            .scan(
                (0usize, 0usize),
                move |(invalidated, to_skip), (i, (current, previous))| {
                    let emit =
                        !current.skip && (current != previous || *invalidated > 0) && *to_skip == 0;

                    let result = if emit {
                        let x = u16::try_from(i % width).unwrap_or(0);
                        let y = u16::try_from(i / width).unwrap_or(0);
                        Some((x, y, current))
                    } else {
                        None
                    };

                    *to_skip = current.width().saturating_sub(1);
                    let affected = current.width().max(previous.width());
                    *invalidated = (*invalidated).max(affected).saturating_sub(1);

                    Some(result)
                },
            )
            .flatten()
    }

    const fn index(&self, position: Position) -> usize {
        (position.y as usize) * (self.size.width as usize) + (position.x as usize)
    }

    const fn within_bounds(&self, position: Position) -> bool {
        position.x < self.size.width && position.y < self.size.height
    }
}

pub struct BufferView {
    area: Rect,
}

impl BufferView {
    pub const fn area(&self) -> Rect {
        self.area
    }

    pub const fn width(&self) -> u16 {
        self.area.width
    }

    pub const fn height(&self) -> u16 {
        self.area.height
    }

    pub fn set(&self, buffer: &mut Buffer, x: u16, y: u16, cell: Cell) {
        let position = Position::new(x, y);

        if !self.within_bounds(position) {
            return;
        }

        buffer.set(self.area + position, cell);
    }

    pub const fn subview(&self, area: Rect) -> Self {
        Self {
            area: Rect::new(
                self.area.x + area.x,
                self.area.y + area.y,
                area.width,
                area.height,
            ),
        }
    }

    pub fn fill(&self, buffer: &mut Buffer, style: &Style) {
        for y in 0..self.height() {
            for x in 0..self.width() {
                if let Some(cell) = buffer.get_mut(self.area + Position::new(x, y)) {
                    cell.background = style.background;
                }
            }
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.width() == 0 && self.height() == 0
    }

    const fn within_bounds(&self, position: Position) -> bool {
        position.x < self.area.width && position.y < self.area.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use korin_style::Color;

    #[test]
    fn new_buffer_has_correct_size() {
        let buf = Buffer::new(10, 5);
        assert_eq!(buf.size().width, 10);
        assert_eq!(buf.size().height, 5);
        assert_eq!(buf.cells().len(), 50);
    }

    #[test]
    fn new_buffer_is_empty_cells() {
        let buf = Buffer::new(3, 3);
        for cell in buf.cells() {
            assert_eq!(cell.symbol, ' ');
        }
    }

    #[test]
    fn get_returns_cell() {
        let buf = Buffer::new(5, 5);
        let cell = buf.get(Position::new(2, 2));
        assert!(cell.is_some());
    }

    #[test]
    fn get_out_of_bounds_returns_none() {
        let buf = Buffer::new(5, 5);
        assert!(buf.get(Position::new(5, 0)).is_none());
        assert!(buf.get(Position::new(0, 5)).is_none());
        assert!(buf.get(Position::new(10, 10)).is_none());
    }

    #[test]
    fn set_modifies_cell() {
        let mut buf = Buffer::new(5, 5);
        buf.set(Position::new(1, 1), Cell::new('X'));

        let cell = buf.get(Position::new(1, 1)).expect("failed");
        assert_eq!(cell.symbol, 'X');
    }

    #[test]
    fn set_out_of_bounds_is_noop() {
        let mut buf = Buffer::new(5, 5);
        buf.set(Position::new(10, 10), Cell::new('X'));
    }

    #[test]
    fn clear_resets_all_cells() {
        let mut buf = Buffer::new(3, 3);
        buf.set(Position::new(0, 0), Cell::new('A'));
        buf.set(Position::new(1, 1), Cell::new('B'));

        buf.clear();

        for cell in buf.cells() {
            assert_eq!(cell.symbol, ' ');
        }
    }

    #[test]
    fn resize_changes_dimensions() {
        let mut buf = Buffer::new(5, 5);
        buf.resize(Size::new(10, 3));

        assert_eq!(buf.size().width, 10);
        assert_eq!(buf.size().height, 3);
        assert_eq!(buf.cells().len(), 30);
    }

    #[test]
    fn diff_detects_changes() {
        let mut current = Buffer::new(3, 1);
        let previous = Buffer::new(3, 1);

        current.set(Position::new(1, 0), Cell::new('X'));

        let changes: Vec<_> = current.diff(&previous).collect();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0], (1, 0, &Cell::new('X')));
    }

    #[test]
    fn diff_empty_when_identical() {
        let buf1 = Buffer::new(3, 3);
        let buf2 = Buffer::new(3, 3);

        assert!(buf1.diff(&buf2).next().is_none());
    }

    #[test]
    fn view_has_full_dimensions() {
        let buf = Buffer::new(10, 5);
        let view = buf.view();

        assert_eq!(view.width(), 10);
        assert_eq!(view.height(), 5);
    }

    #[test]
    fn view_set_writes_to_buffer() {
        let mut buf = Buffer::new(5, 5);
        let view = buf.view();

        view.set(&mut buf, 2, 2, Cell::new('O'));

        assert_eq!(buf.get(Position::new(2, 2)).expect("failed").symbol, 'O');
    }

    #[test]
    fn subview_offsets_correctly() {
        let mut buf = Buffer::new(10, 10);
        let view = buf.view();
        let sub = view.subview(Rect::new(3, 3, 4, 4));

        sub.set(&mut buf, 0, 0, Cell::new('X'));

        assert_eq!(buf.get(Position::new(3, 3)).expect("failed").symbol, 'X');
    }

    #[test]
    fn subview_clips_to_bounds() {
        let mut buf = Buffer::new(10, 10);
        let view = buf.view();
        let sub = view.subview(Rect::new(5, 5, 3, 3));

        sub.set(&mut buf, 10, 10, Cell::new('X'));

        assert_eq!(buf.get(Position::new(5, 5)).expect("failed").symbol, ' ');
    }

    #[test]
    fn fill_sets_background() {
        let mut buf = Buffer::new(3, 3);
        let view = buf.view();
        let style = Style::new().background(Color::Red);

        view.fill(&mut buf, &style);

        for cell in buf.cells() {
            assert_eq!(cell.background, Color::Red);
        }
    }

    #[test]
    fn nested_subviews() {
        let mut buf = Buffer::new(20, 20);
        let view = buf.view();
        let sub1 = view.subview(Rect::new(5, 5, 10, 10));
        let sub2 = sub1.subview(Rect::new(2, 2, 4, 4));

        sub2.set(&mut buf, 1, 1, Cell::new('Z'));

        // 5 + 2 + 1 = 8
        assert_eq!(buf.get(Position::new(8, 8)).expect("failed").symbol, 'Z');
    }
}
