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
        let area = Rect::new(0, 0, self.size.width, self.size.height);
        BufferView { area, clip: area }
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
    clip: Rect,
}

impl BufferView {
    pub const fn area(&self) -> Rect {
        self.area
    }

    pub const fn clip(&self) -> Rect {
        self.clip
    }

    pub const fn width(&self) -> u16 {
        self.area.width
    }

    pub const fn height(&self) -> u16 {
        self.area.height
    }

    pub fn set(&self, buffer: &mut Buffer, x: u16, y: u16, cell: Cell) {
        let abs_x = self.area.x + x;
        let abs_y = self.area.y + y;

        if !self.within_clip(abs_x, abs_y) {
            return;
        }

        buffer.set(Position::new(abs_x, abs_y), cell);
    }

    pub const fn subview(area: Rect, clip: Rect) -> Self {
        Self { area, clip }
    }

    pub fn fill(&self, buffer: &mut Buffer, style: &Style) {
        for y in 0..self.height() {
            for x in 0..self.width() {
                let abs_x = self.area.x + x;
                let abs_y = self.area.y + y;

                if !self.within_clip(abs_x, abs_y) {
                    continue;
                }

                if let Some(cell) = buffer.get_mut(Position::new(abs_x, abs_y)) {
                    cell.background = style.background();
                }
            }
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.width() == 0 && self.height() == 0
    }

    const fn within_clip(&self, abs_x: u16, abs_y: u16) -> bool {
        abs_x >= self.clip.x
            && abs_x < self.clip.x + self.clip.width
            && abs_y >= self.clip.y
            && abs_y < self.clip.y + self.clip.height
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
        let sub = BufferView::subview(Rect::new(3, 3, 4, 4), Rect::new(3, 3, 4, 4));

        sub.set(&mut buf, 0, 0, Cell::new('X'));

        assert_eq!(buf.get(Position::new(3, 3)).expect("failed").symbol, 'X');
    }

    #[test]
    fn subview_clips_to_bounds() {
        let mut buf = Buffer::new(10, 10);
        let clip = Rect::new(5, 5, 3, 3);
        let sub = BufferView::subview(Rect::new(5, 5, 3, 3), clip);

        sub.set(&mut buf, 10, 10, Cell::new('X'));

        assert_eq!(buf.get(Position::new(5, 5)).expect("failed").symbol, ' ');
    }

    #[test]
    fn fill_sets_background() {
        let mut buf = Buffer::new(3, 3);
        let view = buf.view();
        let style = Style::builder().background(Color::Red).build();

        view.fill(&mut buf, &style);

        for cell in buf.cells() {
            assert_eq!(cell.background, Color::Red);
        }
    }

    #[test]
    fn clip_restricts_rendering() {
        let mut buf = Buffer::new(20, 20);
        let area = Rect::new(5, 5, 10, 10);
        let clip = Rect::new(5, 5, 3, 3); // smaller than area
        let sub = BufferView::subview(area, clip);

        // inside clip
        sub.set(&mut buf, 0, 0, Cell::new('A'));
        assert_eq!(buf.get(Position::new(5, 5)).expect("failed").symbol, 'A');

        // outside clip but inside area
        sub.set(&mut buf, 5, 5, Cell::new('B'));
        assert_eq!(buf.get(Position::new(10, 10)).expect("failed").symbol, ' ');
    }

    #[test]
    fn fill_respects_clip() {
        let mut buf = Buffer::new(10, 10);
        let area = Rect::new(0, 0, 10, 10);
        let clip = Rect::new(2, 2, 3, 3);
        let sub = BufferView::subview(area, clip);
        let style = Style::builder().background(Color::Red).build();

        sub.fill(&mut buf, &style);

        // inside clip - should be red
        assert_eq!(
            buf.get(Position::new(3, 3)).expect("failed").background,
            Color::Red
        );

        // outside clip - should be default
        assert_eq!(
            buf.get(Position::new(0, 0)).expect("failed").background,
            Color::Reset
        );
        assert_eq!(
            buf.get(Position::new(8, 8)).expect("failed").background,
            Color::Reset
        );
    }
}
