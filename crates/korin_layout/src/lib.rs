use taffy::{Dimension, LengthPercentage};

mod engine;
mod error;
mod geometry;
mod layout;

pub use engine::Engine as LayoutEngine;
pub use error::{LayoutError, LayoutResult};
pub use geometry::{Point, Rect, Size};
pub use layout::Layout;

#[must_use]
pub fn layout() -> Layout {
    Layout::new()
}

#[must_use]
pub fn row() -> Layout {
    layout().row()
}

#[must_use]
pub fn col() -> Layout {
    layout().col()
}

#[must_use]
pub const fn len(val: f32) -> LengthPercentage {
    LengthPercentage::length(val)
}

#[must_use]
pub const fn pct(val: f32) -> LengthPercentage {
    LengthPercentage::percent(val)
}

#[must_use]
pub const fn full() -> Dimension {
    Dimension::percent(1.0)
}
