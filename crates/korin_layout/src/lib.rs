mod style;

pub use style::LayoutStyle;
use taffy::LengthPercentage;

#[must_use]
pub fn layout() -> LayoutStyle {
    LayoutStyle::new()
}

#[must_use]
pub fn row() -> LayoutStyle {
    layout().row()
}

#[must_use]
pub fn col() -> LayoutStyle {
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
