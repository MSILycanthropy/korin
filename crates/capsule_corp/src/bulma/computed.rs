use crate::{
    AlignItems, AlignSelf, BorderStyle, Color, Dimension, Display, Edges, FlexDirection, FlexWrap,
    FontStyle, FontWeight, JustifyContent, Length, Overflow, OverflowWrap, TextAlign,
    TextDecoration, VerticalAlign, Visibility, WhiteSpace,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub display: Display,

    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,

    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Dimension,
    pub align_self: AlignSelf,

    pub row_gap: Length,
    pub column_gap: Length,

    pub width: Dimension,
    pub height: Dimension,
    pub min_width: Dimension,
    pub max_width: Dimension,
    pub min_height: Dimension,
    pub max_height: Dimension,

    pub margin: Edges<Length>,
    pub padding: Edges<Length>,

    pub border_style: Edges<BorderStyle>,
    pub border_color: Edges<Color>,

    pub color: Color,
    pub background_color: Color,

    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub text_decoration: TextDecoration,
    pub text_align: TextAlign,
    pub vertical_align: VerticalAlign,
    pub white_space: WhiteSpace,
    pub overflow_wrap: OverflowWrap,

    pub overflow_x: Overflow,
    pub overflow_y: Overflow,

    pub visibility: Visibility,
    pub z_index: i16,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            display: Display::default(),

            flex_direction: FlexDirection::default(),
            flex_wrap: FlexWrap::default(),
            justify_content: JustifyContent::default(),
            align_items: AlignItems::default(),

            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Dimension::Auto,
            align_self: AlignSelf::default(),

            row_gap: Length::ZERO,
            column_gap: Length::ZERO,

            width: Dimension::Auto,
            height: Dimension::Auto,
            min_width: Dimension::Auto,
            max_width: Dimension::None,
            min_height: Dimension::Auto,
            max_height: Dimension::None,

            margin: Edges::default(),
            padding: Edges::default(),

            border_style: Edges::default(),
            border_color: Edges::all(Color::Reset),

            color: Color::Reset,
            background_color: Color::Reset,

            font_weight: FontWeight::default(),
            font_style: FontStyle::default(),
            text_decoration: TextDecoration::default(),
            text_align: TextAlign::default(),
            vertical_align: VerticalAlign::default(),
            white_space: WhiteSpace::default(),
            overflow_wrap: OverflowWrap::default(),

            overflow_x: Overflow::default(),
            overflow_y: Overflow::default(),

            visibility: Visibility::default(),
            z_index: 0,
        }
    }
}

impl ComputedStyle {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn inherit_from(parent: &Self) -> Self {
        Self {
            color: parent.color,
            font_weight: parent.font_weight,
            font_style: parent.font_style,
            text_decoration: parent.text_decoration,
            text_align: parent.text_align,
            white_space: parent.white_space,
            overflow_wrap: parent.overflow_wrap,
            visibility: parent.visibility,
            ..Self::default()
        }
    }

    #[must_use]
    pub const fn is_flex_container(&self) -> bool {
        matches!(self.display, Display::Flex)
    }

    #[must_use]
    pub const fn is_grid_container(&self) -> bool {
        matches!(self.display, Display::Grid)
    }

    #[must_use]
    pub const fn is_inline_container(&self) -> bool {
        matches!(self.display, Display::Inline)
    }

    #[must_use]
    pub const fn is_hidden(&self) -> bool {
        matches!(self.display, Display::None) || matches!(self.visibility, Visibility::Hidden)
    }
}
