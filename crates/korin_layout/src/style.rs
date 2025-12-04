use taffy::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap,
    JustifyContent, LengthPercentage, LengthPercentageAuto, Rect, Size, Style,
};

#[derive(Clone, Default)]
pub struct LayoutStyle(pub Style);
impl LayoutStyle {
    #[must_use]
    pub fn new() -> Self {
        Self(Style::default())
    }

    #[must_use]
    pub const fn row(mut self) -> Self {
        self.0.display = Display::Flex;
        self.0.flex_direction = FlexDirection::Row;
        self
    }

    #[must_use]
    pub const fn col(mut self) -> Self {
        self.0.display = Display::Flex;
        self.0.flex_direction = FlexDirection::Column;
        self
    }

    #[must_use]
    pub const fn grow(mut self, v: f32) -> Self {
        self.0.flex_grow = v;
        self
    }

    #[must_use]
    pub const fn shrink(mut self, v: f32) -> Self {
        self.0.flex_shrink = v;
        self
    }

    #[must_use]
    pub fn basis(mut self, v: impl Into<Dimension>) -> Self {
        self.0.flex_basis = v.into();
        self
    }

    #[must_use]
    pub const fn wrap(mut self) -> Self {
        self.0.flex_wrap = FlexWrap::Wrap;
        self
    }

    #[must_use]
    pub fn gap(mut self, v: impl Into<LengthPercentage> + Copy) -> Self {
        self.0.gap = Size {
            width: v.into(),
            height: v.into(),
        };
        self
    }

    #[must_use]
    pub fn gap_x(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.gap.width = v.into();
        self
    }

    #[must_use]
    pub fn gap_y(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.gap.height = v.into();
        self
    }

    #[must_use]
    pub fn p(mut self, v: impl Into<LengthPercentage> + Copy) -> Self {
        let val = v.into();
        self.0.padding = Rect {
            left: val,
            right: val,
            top: val,
            bottom: val,
        };
        self
    }

    #[must_use]
    pub fn px(mut self, v: impl Into<LengthPercentage> + Copy) -> Self {
        let val = v.into();
        self.0.padding.left = val;
        self.0.padding.right = val;
        self
    }

    #[must_use]
    pub fn py(mut self, v: impl Into<LengthPercentage> + Copy) -> Self {
        let val = v.into();
        self.0.padding.top = val;
        self.0.padding.bottom = val;
        self
    }

    #[must_use]
    pub fn pt(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.padding.top = v.into();
        self
    }

    #[must_use]
    pub fn pb(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.padding.bottom = v.into();
        self
    }

    #[must_use]
    pub fn pl(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.padding.left = v.into();
        self
    }

    #[must_use]
    pub fn pr(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.padding.right = v.into();
        self
    }

    #[must_use]
    pub fn m(mut self, v: impl Into<LengthPercentageAuto> + Copy) -> Self {
        let val = v.into();
        self.0.margin = Rect {
            left: val,
            right: val,
            top: val,
            bottom: val,
        };
        self
    }

    #[must_use]
    pub fn mx(mut self, v: impl Into<LengthPercentageAuto> + Copy) -> Self {
        let val = v.into();
        self.0.margin.left = val;
        self.0.margin.right = val;
        self
    }

    #[must_use]
    pub fn my(mut self, v: impl Into<LengthPercentageAuto> + Copy) -> Self {
        let val = v.into();
        self.0.margin.top = val;
        self.0.margin.bottom = val;
        self
    }

    #[must_use]
    pub fn mt(mut self, v: impl Into<LengthPercentageAuto>) -> Self {
        self.0.margin.top = v.into();
        self
    }

    #[must_use]
    pub fn mb(mut self, v: impl Into<LengthPercentageAuto>) -> Self {
        self.0.margin.bottom = v.into();
        self
    }

    #[must_use]
    pub fn ml(mut self, v: impl Into<LengthPercentageAuto>) -> Self {
        self.0.margin.left = v.into();
        self
    }

    #[must_use]
    pub fn mr(mut self, v: impl Into<LengthPercentageAuto>) -> Self {
        self.0.margin.right = v.into();
        self
    }

    #[must_use]
    pub fn w(mut self, v: impl Into<Dimension>) -> Self {
        self.0.size.width = v.into();
        self
    }

    #[must_use]
    pub fn h(mut self, v: impl Into<Dimension>) -> Self {
        self.0.size.height = v.into();
        self
    }

    #[must_use]
    pub fn min_w(mut self, v: impl Into<Dimension>) -> Self {
        self.0.min_size.width = v.into();
        self
    }

    #[must_use]
    pub fn min_h(mut self, v: impl Into<Dimension>) -> Self {
        self.0.min_size.height = v.into();
        self
    }

    #[must_use]
    pub fn max_w(mut self, v: impl Into<Dimension>) -> Self {
        self.0.max_size.width = v.into();
        self
    }

    #[must_use]
    pub fn max_h(mut self, v: impl Into<Dimension>) -> Self {
        self.0.max_size.height = v.into();
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

    #[must_use]
    pub fn build(self) -> Style {
        self.0
    }
}

impl From<LayoutStyle> for Style {
    fn from(value: LayoutStyle) -> Self {
        value.0
    }
}
