use crate::{ComputedStyle, brief::box_model::SizeConstraints};

#[must_use]
pub fn resolve_size_constraints(
    style: &ComputedStyle,
    parent_width: u16,
    parent_height: Option<u16>,
) -> SizeConstraints {
    SizeConstraints {
        width: style.width.resolve(parent_width),
        height: style.height.resolve(parent_height.unwrap_or(0)),
        min_width: style.min_width.resolve(parent_width).unwrap_or(0),
        max_width: style.max_width.resolve(parent_width),
        min_height: parent_height.map_or(0, |h| style.min_height.resolve(h).unwrap_or(0)),
        max_height: parent_height.and_then(|h| style.max_height.resolve(h)),
    }
}
