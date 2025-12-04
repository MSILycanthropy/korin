mod geometry;
mod layout;

pub use geometry::{Point, Rect, Size};
pub use layout::Layout;
use taffy::LengthPercentage;

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
