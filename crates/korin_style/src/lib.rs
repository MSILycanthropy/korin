use korin_layout::{
    AlignContent, AlignItems, AlignSelf, Display, IntoDimension, IntoF32, IntoLengthPercentage,
    IntoLengthPercentageAuto, IntoTracks, JustifyContent, Layout, Overflow, Position,
};

use crate::appearance::Appearance;
pub use crate::{
    border::{BorderStyle, Borders},
    color::Color,
    text::{Alignment, Modifiers},
};

mod appearance;
mod border;
mod color;
mod text;

#[derive(Clone, Debug, Default)]
pub struct Style {
    appearance: Appearance,
    layout: Layout,
}

impl Style {
    #[must_use]
    pub fn builder() -> StyleBuilder {
        StyleBuilder::default()
    }

    #[must_use]
    pub const fn appearance(&self) -> &Appearance {
        &self.appearance
    }

    #[must_use]
    pub const fn layout(&self) -> &Layout {
        &self.layout
    }

    #[must_use]
    pub const fn background(&self) -> Color {
        self.appearance.background
    }

    #[must_use]
    pub const fn text_color(&self) -> Color {
        self.appearance.text_color
    }

    #[must_use]
    pub const fn borders(&self) -> Borders {
        self.appearance.borders
    }

    #[must_use]
    pub const fn border_style(&self) -> BorderStyle {
        self.appearance.border_style
    }

    #[must_use]
    pub const fn border_color(&self) -> Color {
        self.appearance.border_color
    }

    #[must_use]
    pub const fn text_alignment(&self) -> Alignment {
        self.appearance.text_alignment
    }

    #[must_use]
    pub const fn text_modifiers(&self) -> Modifiers {
        self.appearance.text_modifiers
    }

    #[must_use]
    pub const fn z_index(&self) -> i32 {
        self.appearance.z_index
    }

    #[must_use]
    pub const fn overflow_x(&self) -> Overflow {
        self.layout.style.overflow.x
    }

    #[must_use]
    pub const fn overflow_y(&self) -> Overflow {
        self.layout.style.overflow.y
    }

    #[must_use]
    pub fn merge(mut self, parent: &Self) -> Self {
        self.appearance = self.appearance.merge(&parent.appearance);
        self
    }
}

#[derive(Clone, Debug, Default)]
pub struct StyleBuilder {
    appearance: Appearance,
    layout: Layout,
}

impl StyleBuilder {
    #[must_use]
    pub fn build(self) -> Style {
        Style {
            appearance: self.appearance,
            layout: self.layout,
        }
    }

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn with_appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = appearance;
        self
    }

    #[must_use]
    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    // === Appearance delegates ===

    #[must_use]
    pub const fn text_color(mut self, color: Color) -> Self {
        self.appearance = self.appearance.text_color(color);
        self
    }

    #[must_use]
    pub const fn background(mut self, color: Color) -> Self {
        self.appearance = self.appearance.background(color);
        self
    }

    #[must_use]
    pub const fn borders(mut self, borders: Borders) -> Self {
        self.appearance = self.appearance.borders(borders);
        self
    }

    #[must_use]
    pub const fn border_style(mut self, style: BorderStyle) -> Self {
        self.appearance = self.appearance.border_style(style);
        self
    }

    #[must_use]
    pub const fn border_color(mut self, color: Color) -> Self {
        self.appearance = self.appearance.border_color(color);
        self
    }

    #[must_use]
    pub const fn text_alignment(mut self, alignment: Alignment) -> Self {
        self.appearance = self.appearance.text_alignment(alignment);
        self
    }

    #[must_use]
    pub const fn text_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.appearance = self.appearance.text_modifiers(modifiers);
        self
    }

    #[must_use]
    pub const fn bordered(self) -> Self {
        self.borders(Borders::ALL)
    }

    #[must_use]
    pub const fn text_bold(self) -> Self {
        let modifiers = self.appearance.text_modifiers;
        self.text_modifiers(modifiers.union(Modifiers::BOLD))
    }

    #[must_use]
    pub const fn text_italic(self) -> Self {
        let modifiers = self.appearance.text_modifiers;
        self.text_modifiers(modifiers.union(Modifiers::ITALIC))
    }

    #[must_use]
    pub const fn text_underline(self) -> Self {
        let modifiers = self.appearance.text_modifiers;
        self.text_modifiers(modifiers.union(Modifiers::UNDERLINE))
    }

    #[must_use]
    pub const fn text_dim(self) -> Self {
        let modifiers = self.appearance.text_modifiers;
        self.text_modifiers(modifiers.union(Modifiers::DIM))
    }

    #[must_use]
    pub const fn text_centered(self) -> Self {
        self.text_alignment(Alignment::Center)
    }

    #[must_use]
    pub const fn text_right(self) -> Self {
        self.text_alignment(Alignment::Right)
    }

    // === Layout delegates ===

    #[must_use]
    pub fn row(mut self) -> Self {
        self.layout = self.layout.flex_row();
        self
    }

    #[must_use]
    pub fn col(mut self) -> Self {
        self.layout = self.layout.flex_col();
        self
    }

    #[must_use]
    pub fn grid(mut self) -> Self {
        self.layout = self.layout.display_grid();
        self
    }

    #[must_use]
    pub fn grow(mut self, v: impl IntoF32) -> Self {
        self.layout = self.layout.grow(v);
        self
    }

    #[must_use]
    pub fn shrink(mut self, v: impl IntoF32) -> Self {
        self.layout = self.layout.shrink(v);
        self
    }

    #[must_use]
    pub fn basis(mut self, v: impl IntoDimension) -> Self {
        self.layout = self.layout.basis(v);
        self
    }

    #[must_use]
    pub fn wrap(mut self) -> Self {
        self.layout = self.layout.wrap();
        self
    }

    #[must_use]
    pub fn gap(mut self, v: impl IntoLengthPercentage + Copy) -> Self {
        self.layout = self.layout.gap(v);
        self
    }

    #[must_use]
    pub fn gap_x(mut self, v: impl IntoLengthPercentage) -> Self {
        self.layout = self.layout.gap_x(v);
        self
    }

    #[must_use]
    pub fn gap_y(mut self, v: impl IntoLengthPercentage) -> Self {
        self.layout = self.layout.gap_y(v);
        self
    }

    #[must_use]
    pub fn p(mut self, v: impl IntoLengthPercentage + Copy) -> Self {
        self.layout = self.layout.p(v);
        self
    }

    #[must_use]
    pub fn px(mut self, v: impl IntoLengthPercentage + Copy) -> Self {
        self.layout = self.layout.px(v);
        self
    }

    #[must_use]
    pub fn py(mut self, v: impl IntoLengthPercentage + Copy) -> Self {
        self.layout = self.layout.py(v);
        self
    }

    #[must_use]
    pub fn pt(mut self, v: impl IntoLengthPercentage) -> Self {
        self.layout = self.layout.pt(v);
        self
    }

    #[must_use]
    pub fn pb(mut self, v: impl IntoLengthPercentage) -> Self {
        self.layout = self.layout.pb(v);
        self
    }

    #[must_use]
    pub fn pl(mut self, v: impl IntoLengthPercentage) -> Self {
        self.layout = self.layout.pl(v);
        self
    }

    #[must_use]
    pub fn pr(mut self, v: impl IntoLengthPercentage) -> Self {
        self.layout = self.layout.pr(v);
        self
    }

    #[must_use]
    pub fn m(mut self, v: impl IntoLengthPercentageAuto + Copy) -> Self {
        self.layout = self.layout.m(v);
        self
    }

    #[must_use]
    pub fn mx(mut self, v: impl IntoLengthPercentageAuto + Copy) -> Self {
        self.layout = self.layout.mx(v);
        self
    }

    #[must_use]
    pub fn my(mut self, v: impl IntoLengthPercentageAuto + Copy) -> Self {
        self.layout = self.layout.my(v);
        self
    }

    #[must_use]
    pub fn mt(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.layout = self.layout.mt(v);
        self
    }

    #[must_use]
    pub fn mb(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.layout = self.layout.mb(v);
        self
    }

    #[must_use]
    pub fn ml(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.layout = self.layout.ml(v);
        self
    }

    #[must_use]
    pub fn mr(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.layout = self.layout.mr(v);
        self
    }

    #[must_use]
    pub fn w(mut self, v: impl IntoDimension) -> Self {
        self.layout = self.layout.w(v);
        self
    }

    #[must_use]
    pub fn h(mut self, v: impl IntoDimension) -> Self {
        self.layout = self.layout.h(v);
        self
    }

    #[must_use]
    pub fn min_w(mut self, v: impl IntoDimension) -> Self {
        self.layout = self.layout.min_w(v);
        self
    }

    #[must_use]
    pub fn min_h(mut self, v: impl IntoDimension) -> Self {
        self.layout = self.layout.min_h(v);
        self
    }

    #[must_use]
    pub fn max_w(mut self, v: impl IntoDimension) -> Self {
        self.layout = self.layout.max_w(v);
        self
    }

    #[must_use]
    pub fn max_h(mut self, v: impl IntoDimension) -> Self {
        self.layout = self.layout.max_h(v);
        self
    }

    #[must_use]
    pub fn justify(mut self, v: JustifyContent) -> Self {
        self.layout = self.layout.justify(v);
        self
    }

    #[must_use]
    pub fn items(mut self, v: AlignItems) -> Self {
        self.layout = self.layout.items(v);
        self
    }

    #[must_use]
    pub fn content(mut self, v: AlignContent) -> Self {
        self.layout = self.layout.content(v);
        self
    }

    #[must_use]
    pub fn align_self(mut self, v: AlignSelf) -> Self {
        self.layout = self.layout.align_self(v);
        self
    }

    #[must_use]
    pub fn cols(mut self, tracks: impl IntoTracks) -> Self {
        self.layout = self.layout.cols(tracks);
        self
    }

    #[must_use]
    pub fn rows(mut self, tracks: impl IntoTracks) -> Self {
        self.layout = self.layout.rows(tracks);
        self
    }

    #[must_use]
    pub fn col_start(mut self, v: i16) -> Self {
        self.layout = self.layout.col_start(v);
        self
    }

    #[must_use]
    pub fn col_end(mut self, v: i16) -> Self {
        self.layout = self.layout.col_end(v);
        self
    }

    #[must_use]
    pub fn col_span(mut self, v: u16) -> Self {
        self.layout = self.layout.col_span(v);
        self
    }

    #[must_use]
    pub fn row_start(mut self, v: i16) -> Self {
        self.layout = self.layout.row_start(v);
        self
    }

    #[must_use]
    pub fn row_end(mut self, v: i16) -> Self {
        self.layout = self.layout.row_end(v);
        self
    }

    #[must_use]
    pub fn row_span(mut self, v: u16) -> Self {
        self.layout = self.layout.row_span(v);
        self
    }

    #[must_use]
    pub const fn z_index(mut self, z: i32) -> Self {
        self.appearance = self.appearance.z_index(z);
        self
    }

    #[must_use]
    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.layout = self.layout.overflow(overflow);
        self
    }

    #[must_use]
    pub fn overflow_x(mut self, overflow: Overflow) -> Self {
        self.layout = self.layout.overflow_x(overflow);
        self
    }

    #[must_use]
    pub fn overflow_y(mut self, overflow: Overflow) -> Self {
        self.layout = self.layout.overflow_y(overflow);
        self
    }

    #[must_use]
    pub fn top(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.layout = self.layout.top(v);
        self
    }

    #[must_use]
    pub fn bottom(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.layout = self.layout.bottom(v);
        self
    }

    #[must_use]
    pub fn left(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.layout = self.layout.left(v);
        self
    }

    #[must_use]
    pub fn right(mut self, v: impl IntoLengthPercentageAuto) -> Self {
        self.layout = self.layout.right(v);
        self
    }

    #[must_use]
    pub fn position(mut self, position: Position) -> Self {
        self.layout = self.layout.position(position);
        self
    }

    #[must_use]
    pub fn display(mut self, display: Display) -> Self {
        self.layout = self.layout.display(display);
        self
    }

    #[must_use]
    pub fn merge(mut self, parent: &Self) -> Self {
        self.appearance = self.appearance.merge(&parent.appearance);
        self
    }
}

impl From<Layout> for Style {
    fn from(layout: Layout) -> Self {
        Self {
            layout,
            appearance: Appearance::default(),
        }
    }
}

impl From<Appearance> for Style {
    fn from(appearance: Appearance) -> Self {
        Self {
            appearance,
            layout: Layout::default(),
        }
    }
}
