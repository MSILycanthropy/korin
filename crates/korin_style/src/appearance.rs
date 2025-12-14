use std::fmt;

use crate::{Alignment, BorderStyle, Borders, Color, Modifiers, WhiteSpace};

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Appearance {
    pub background: Color,
    pub borders: Borders,
    pub border_style: BorderStyle,
    pub border_color: Color,
    pub text_color: Color,
    pub text_alignment: Alignment,
    pub text_modifiers: Modifiers,
    pub z_index: i32,
    pub white_space: WhiteSpace,
}

impl fmt::Debug for Appearance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("Appearance");

        if self.background != Color::Reset {
            s.field("background", &self.background);
        }

        if self.text_color != Color::Reset {
            s.field("foreground", &self.text_color);
        }

        if !self.borders.is_empty() {
            s.field("borders", &self.borders);
        }

        if self.border_style != BorderStyle::Plain {
            s.field("border_style", &self.border_style);
        }

        if self.border_color != Color::Reset {
            s.field("border_color", &self.border_color);
        }

        if self.text_alignment != Alignment::Left {
            s.field("align", &self.text_alignment);
        }

        if !self.text_modifiers.is_empty() {
            s.field("modifiers", &self.text_modifiers);
        }

        if self.z_index != 0 {
            s.field("z", &self.z_index);
        }

        s.field("whitepsace", &self.white_space);

        s.finish()
    }
}

impl Appearance {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            background: Color::Reset,
            text_color: Color::Reset,
            borders: Borders::empty(),
            border_style: BorderStyle::Plain,
            border_color: Color::Reset,
            text_alignment: Alignment::Left,
            text_modifiers: Modifiers::empty(),
            z_index: 0,
            white_space: WhiteSpace::Normal,
        }
    }

    #[must_use]
    pub fn inherit(mut self, parent: &Self) -> Self {
        if self.text_color == Color::Reset {
            self.text_color = parent.text_color;
        }

        if self.text_modifiers.is_empty() {
            self.text_modifiers = parent.text_modifiers;
        }

        if self.text_alignment == Alignment::Left && parent.text_alignment != Alignment::Left {
            self.text_alignment = parent.text_alignment;
        }

        if self.background == Color::Reset {
            self.background = parent.background;
        }

        if parent.white_space == WhiteSpace::NoWrap {
            self.white_space = parent.white_space;
        }

        self
    }

    #[must_use]
    pub fn merge(&self, base: &Self) -> Self {
        Self {
            text_color: pick(self.text_color, base.text_color),
            background: pick(self.background, base.background),
            borders: pick(self.borders, base.borders),
            border_style: pick(self.border_style, base.border_style),
            border_color: pick(self.border_color, base.border_color),
            text_alignment: pick(self.text_alignment, base.text_alignment),
            text_modifiers: pick(self.text_modifiers, base.text_modifiers),
            z_index: pick(self.z_index, base.z_index),
            white_space: pick(self.white_space, base.white_space),
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn pick<T: PartialEq + Default>(value: T, base: T) -> T {
    if value == T::default() { base } else { value }
}
