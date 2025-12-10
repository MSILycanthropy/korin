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
