mod buffer;
mod cell;
mod event;
mod renderer;
mod symbols;
mod terminal;

pub mod prelude;

pub mod crossterm {
    pub use crossterm::*;
}

pub use buffer::Buffer;
pub use cell::Cell;
pub use event::{Event, dispatch, poll};
pub use renderer::render;
pub use terminal::Terminal;

pub type Rect = korin_geometry::Rect<u16>;
pub type Size = korin_geometry::Size<u16>;
pub type Position = korin_geometry::Point<u16>;
