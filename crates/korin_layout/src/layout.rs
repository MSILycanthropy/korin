use taffy::{
    AlignContent, AlignItems, AlignSelf, Display, FlexDirection, FlexWrap, GridPlacement,
    JustifyContent, Overflow, Rect, Size, Style,
};

use crate::conversions::{
    IntoDimension, IntoF32, IntoLengthPercentage, IntoLengthPercentageAuto, IntoTracks,
};

#[derive(Clone, Debug, Default)]
pub struct Layout(pub Style);

impl Layout {
    #[must_use]
    pub fn new() -> Self {
        Self(Style::default())
    }

    #[must_use]
    pub fn row() -> Self {
        Self::new().flex_row()
    }

    #[must_use]
    pub fn col() -> Self {
        Self::new().flex_col()
    }

    #[must_use]
    pub fn grid() -> Self {
        Self::new().display_grid()
    }

    #[must_use]
    pub const fn display_grid(mut self) -> Self {
        self.0.display = Display::Grid;
        self
    }

    #[must_use]
    pub const fn flex_row(mut self) -> Self {
        self.0.display = Display::Flex;
        self.0.flex_direction = FlexDirection::Row;
        self
    }

    #[must_use]
    pub const fn flex_col(mut self) -> Self {
        self.0.display = Display::Flex;
        self.0.flex_direction = FlexDirection::Column;
        self
    }

    #[must_use]
    pub fn grow(mut self, v: impl IntoF32) -> Self {
        self.0.flex_grow = v.into_f32();
        self
    }

    #[must_use]
    pub fn shrink(mut self, v: impl IntoF32) -> Self {
        self.0.flex_shrink = v.into_f32();
        self
    }

    #[must_use]
    pub fn basis(mut self, v: impl IntoDimension) -> Self {
        self.0.flex_basis = v.into_dimension();
        self
    }

    #[must_use]
    pub const fn wrap(mut self) -> Self {
        self.0.flex_wrap = FlexWrap::Wrap;
        self
    }

    #[must_use]
    pub fn gap(mut self, v: impl IntoLengthPercentage + Copy) -> Self {
        let val = v.into_length_percentage();
        self.0.gap = Size {
            width: val,
            height: val,
        };
        self
    }

    #[must_use]
    pub fn gap_x(mut self, v: impl IntoLengthPercentage) -> Self {
        self.0.gap.width = v.into_length_percentage();
        self
    }

    #[must_use]
    pub fn gap_y(mut self, v: impl IntoLengthPercentage) -> Self {
        self.0.gap.height = v.into_length_percentage();
        self
    }

    #[must_use]
    pub fn p(mut self, v: impl IntoLengthPercentage + Copy) -> Self {
        let val = v.into_length_percentage();
        self.0.padding = Rect {
            left: val,
            right: val,
            top: val,
            bottom: val,
        };
        self
    }

    #[must_use]
    pub fn px(mut self, v: impl IntoLengthPercentage + Copy) -> Self {
        let val = v.into_length_percentage();
        self.0.padding.left = val;
        self.0.padding.right = val;
        self
    }

    #[must_use]
    pub fn py(mut self, v: impl IntoLengthPercentage + Copy) -> Self {
        let val = v.into_length_percentage();
        self.0.padding.top = val;
        self.0.padding.bottom = val;
        self
    }

    #[must_use]
    pub fn pt(mut self, v: impl IntoLengthPercentage) -> Self {
        self.0.padding.top = v.into_length_percentage();
        self
    }

    #[must_use]
    pub fn pb(mut self, v: impl IntoLengthPercentage) -> Self {
        self.0.padding.bottom = v.into_length_percentage();
        self
    }

    #[must_use]
    pub fn pl(mut self, v: impl IntoLengthPercentage) -> Self {
        self.0.padding.left = v.into_length_percentage();
        self
    }

    #[must_use]
    pub fn pr(mut self, v: impl IntoLengthPercentage) -> Self {
        self.0.padding.right = v.into_length_percentage();
        self
    }

    #[must_use]
    pub fn m(mut self, v: impl IntoLengthPercentageAuto + Copy) -> Self {
        let val = v.into_length_percentage_auto();
        self.0.margin = Rect {
            left: val,
            right: val,
            top: val,
            bottom: val,
        };
        self
    }

    #[must_use]
    pub fn mx(mut self, v: impl IntoLengthPercentageAuto + Copy) -> Self {
        let val = v.into_length_percentage_auto();
        self.0.margin.left = val;
        self.0.margin.right = val;
        self
    }

    #[must_use]
    pub fn my(mut self, v: impl IntoLengthPercentageAuto + Copy) -> Self {
        let val = v.into_length_percentage_auto();
        self.0.margin.top = val;
        self.0.margin.bottom = val;
        self
    }

    #[must_use]
    pub fn mt(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.0.margin.top = v.into_length_percentage_auto();
        self
    }

    #[must_use]
    pub fn mb(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.0.margin.bottom = v.into_length_percentage_auto();
        self
    }

    #[must_use]
    pub fn ml(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.0.margin.left = v.into_length_percentage_auto();
        self
    }

    #[must_use]
    pub fn mr(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.0.margin.right = v.into_length_percentage_auto();
        self
    }

    #[must_use]
    pub fn w(mut self, v: impl IntoDimension) -> Self {
        self.0.size.width = v.into_dimension();
        self
    }

    #[must_use]
    pub fn h(mut self, v: impl IntoDimension) -> Self {
        self.0.size.height = v.into_dimension();
        self
    }

    #[must_use]
    pub fn min_w(mut self, v: impl IntoDimension) -> Self {
        self.0.min_size.width = v.into_dimension();
        self
    }

    #[must_use]
    pub fn min_h(mut self, v: impl IntoDimension) -> Self {
        self.0.min_size.height = v.into_dimension();
        self
    }

    #[must_use]
    pub fn max_w(mut self, v: impl IntoDimension) -> Self {
        self.0.max_size.width = v.into_dimension();
        self
    }

    #[must_use]
    pub fn max_h(mut self, v: impl IntoDimension) -> Self {
        self.0.max_size.height = v.into_dimension();
        self
    }

    #[must_use]
    pub const fn justify(mut self, v: JustifyContent) -> Self {
        self.0.justify_content = Some(v);
        self
    }

    #[must_use]
    pub const fn items(mut self, v: AlignItems) -> Self {
        self.0.align_items = Some(v);
        self
    }

    #[must_use]
    pub const fn content(mut self, v: AlignContent) -> Self {
        self.0.align_content = Some(v);
        self
    }

    #[must_use]
    pub const fn align_self(mut self, v: AlignSelf) -> Self {
        self.0.align_self = Some(v);
        self
    }

    // Grid container methods

    #[must_use]
    pub fn cols(mut self, tracks: impl IntoTracks) -> Self {
        self.0.grid_template_columns = tracks.into_tracks();
        self
    }

    #[must_use]
    pub fn rows(mut self, tracks: impl IntoTracks) -> Self {
        self.0.grid_template_rows = tracks.into_tracks();
        self
    }

    // Grid item methods

    #[must_use]
    pub fn col_start(mut self, v: i16) -> Self {
        self.0.grid_column = taffy::Line {
            start: taffy::style_helpers::line(v),
            end: self.0.grid_column.end,
        };
        self
    }

    #[must_use]
    pub fn col_end(mut self, v: i16) -> Self {
        self.0.grid_column = taffy::Line {
            start: self.0.grid_column.start,
            end: taffy::style_helpers::line(v),
        };
        self
    }

    #[must_use]
    pub fn col_span(mut self, v: u16) -> Self {
        self.0.grid_column = taffy::Line {
            start: self.0.grid_column.start,
            end: GridPlacement::Span(v),
        };
        self
    }

    #[must_use]
    pub fn row_start(mut self, v: i16) -> Self {
        self.0.grid_row = taffy::Line {
            start: taffy::style_helpers::line(v),
            end: self.0.grid_row.end,
        };
        self
    }

    #[must_use]
    pub fn row_end(mut self, v: i16) -> Self {
        self.0.grid_row = taffy::Line {
            start: self.0.grid_row.start,
            end: taffy::style_helpers::line(v),
        };
        self
    }

    #[must_use]
    pub fn row_span(mut self, v: u16) -> Self {
        self.0.grid_row = taffy::Line {
            start: self.0.grid_row.start,
            end: GridPlacement::Span(v),
        };
        self
    }

    #[must_use]
    pub const fn overflow(mut self, overflow: Overflow) -> Self {
        self.0.overflow = taffy::Point {
            x: overflow,
            y: overflow,
        };
        self
    }

    #[must_use]
    pub const fn overflow_x(mut self, overflow: Overflow) -> Self {
        self.0.overflow.x = overflow;
        self
    }

    #[must_use]
    pub const fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.0.overflow.y = overflow;
        self
    }

    #[must_use]
    pub fn build(self) -> Style {
        self.0
    }
}

impl From<Layout> for Style {
    fn from(value: Layout) -> Self {
        value.0
    }
}

#[expect(
    unsafe_code,
    reason = "Style is safe as long as calc feature is not used"
)]
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for Layout {}

#[expect(
    unsafe_code,
    reason = "Style is safe as long as calc feature is not used"
)]
unsafe impl Sync for Layout {}
