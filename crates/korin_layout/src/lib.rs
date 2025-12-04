mod style;

pub use style::LayoutStyle;
use taffy::LengthPercentage;

pub fn layout() -> LayoutStyle {
    LayoutStyle::new()
}

pub fn row() -> LayoutStyle {
    layout().row()
}

pub fn col() -> LayoutStyle {
    layout().col()
}

pub fn len(val: f32) -> LengthPercentage {
    LengthPercentage::length(val)
}

pub fn pct(val: f32) -> LengthPercentage {
    LengthPercentage::percent(val)
}
