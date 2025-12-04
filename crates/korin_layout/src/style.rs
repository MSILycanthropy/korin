use taffy::{
    AlignContent, AlignItems, AlignSelf, Dimension, Display, FlexDirection, FlexWrap,
    JustifyContent, LengthPercentage, LengthPercentageAuto, Rect, Size, Style,
};

#[derive(Clone, Default)]
pub struct LayoutStyle(pub Style);
impl LayoutStyle {
    pub fn new() -> Self {
        Self(Style::default())
    }

    pub fn row(mut self) -> Self {
        self.0.display = Display::Flex;
        self.0.flex_direction = FlexDirection::Row;
        self
    }

    pub fn col(mut self) -> Self {
        self.0.display = Display::Flex;
        self.0.flex_direction = FlexDirection::Column;
        self
    }

    pub fn grow(mut self, v: f32) -> Self {
        self.0.flex_grow = v;
        self
    }

    pub fn shrink(mut self, v: f32) -> Self {
        self.0.flex_shrink = v;
        self
    }

    pub fn basis(mut self, v: impl Into<Dimension>) -> Self {
        self.0.flex_basis = v.into();
        self
    }

    pub fn wrap(mut self) -> Self {
        self.0.flex_wrap = FlexWrap::Wrap;
        self
    }

    pub fn gap(mut self, v: impl Into<LengthPercentage> + Copy) -> Self {
        self.0.gap = Size {
            width: v.into(),
            height: v.into(),
        };
        self
    }

    pub fn gap_x(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.gap.width = v.into();
        self
    }

    pub fn gap_y(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.gap.height = v.into();
        self
    }

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

    pub fn px(mut self, v: impl Into<LengthPercentage> + Copy) -> Self {
        let val = v.into();
        self.0.padding.left = val;
        self.0.padding.right = val;
        self
    }

    pub fn py(mut self, v: impl Into<LengthPercentage> + Copy) -> Self {
        let val = v.into();
        self.0.padding.top = val;
        self.0.padding.bottom = val;
        self
    }

    pub fn pt(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.padding.top = v.into();
        self
    }

    pub fn pb(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.padding.bottom = v.into();
        self
    }

    pub fn pl(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.padding.left = v.into();
        self
    }

    pub fn pr(mut self, v: impl Into<LengthPercentage>) -> Self {
        self.0.padding.right = v.into();
        self
    }

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

    pub fn mx(mut self, v: impl Into<LengthPercentageAuto> + Copy) -> Self {
        let val = v.into();
        self.0.margin.left = val;
        self.0.margin.right = val;
        self
    }

    pub fn my(mut self, v: impl Into<LengthPercentageAuto> + Copy) -> Self {
        let val = v.into();
        self.0.margin.top = val;
        self.0.margin.bottom = val;
        self
    }

    pub fn mt(mut self, v: impl Into<LengthPercentageAuto>) -> Self {
        self.0.margin.top = v.into();
        self
    }

    pub fn mb(mut self, v: impl Into<LengthPercentageAuto>) -> Self {
        self.0.margin.bottom = v.into();
        self
    }

    pub fn ml(mut self, v: impl Into<LengthPercentageAuto>) -> Self {
        self.0.margin.left = v.into();
        self
    }

    pub fn mr(mut self, v: impl Into<LengthPercentageAuto>) -> Self {
        self.0.margin.right = v.into();
        self
    }

    pub fn w(mut self, v: impl Into<Dimension>) -> Self {
        self.0.size.width = v.into();
        self
    }

    pub fn h(mut self, v: impl Into<Dimension>) -> Self {
        self.0.size.height = v.into();
        self
    }

    pub fn min_w(mut self, v: impl Into<Dimension>) -> Self {
        self.0.min_size.width = v.into();
        self
    }

    pub fn min_h(mut self, v: impl Into<Dimension>) -> Self {
        self.0.min_size.height = v.into();
        self
    }

    pub fn max_w(mut self, v: impl Into<Dimension>) -> Self {
        self.0.max_size.width = v.into();
        self
    }

    pub fn max_h(mut self, v: impl Into<Dimension>) -> Self {
        self.0.max_size.height = v.into();
        self
    }

    pub fn justify(mut self, v: JustifyContent) -> Self {
        self.0.justify_content = Some(v);
        self
    }

    pub fn items(mut self, v: AlignItems) -> Self {
        self.0.align_items = Some(v);
        self
    }

    pub fn content(mut self, v: AlignContent) -> Self {
        self.0.align_content = Some(v);
        self
    }

    pub fn align_self(mut self, v: AlignSelf) -> Self {
        self.0.align_self = Some(v);
        self
    }

    pub fn build(self) -> Style {
        self.0
    }
}

impl From<LayoutStyle> for Style {
    fn from(value: LayoutStyle) -> Self {
        value.0
    }
}
