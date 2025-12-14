use std::fmt;

use crate::{Alignment, BorderStyle, Borders, Color, Modifiers, WhiteSpace};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
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
    pub const fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    #[must_use]
    pub const fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    #[must_use]
    pub const fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    #[must_use]
    pub const fn border_style(mut self, style: BorderStyle) -> Self {
        self.border_style = style;
        self
    }

    #[must_use]
    pub const fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    #[must_use]
    pub const fn text_alignment(mut self, alignment: Alignment) -> Self {
        self.text_alignment = alignment;
        self
    }

    #[must_use]
    pub const fn text_modifiers(mut self, modifiers: Modifiers) -> Self {
        self.text_modifiers = modifiers;
        self
    }

    #[must_use]
    pub const fn bordered(self) -> Self {
        self.borders(Borders::ALL)
    }

    #[must_use]
    pub const fn text_bold(self) -> Self {
        self.text_modifiers(self.text_modifiers.union(Modifiers::BOLD))
    }

    #[must_use]
    pub const fn text_italic(self) -> Self {
        self.text_modifiers(self.text_modifiers.union(Modifiers::ITALIC))
    }

    #[must_use]
    pub const fn text_underline(self) -> Self {
        self.text_modifiers(self.text_modifiers.union(Modifiers::UNDERLINE))
    }

    #[must_use]
    pub const fn text_dim(self) -> Self {
        self.text_modifiers(self.text_modifiers.union(Modifiers::DIM))
    }

    #[must_use]
    pub const fn text_centered(self) -> Self {
        self.text_alignment(Alignment::Center)
    }

    #[must_use]
    pub const fn text_right(self) -> Self {
        self.text_alignment(Alignment::Right)
    }

    #[must_use]
    pub fn merge(mut self, parent: &Self) -> Self {
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
    pub const fn z_index(mut self, z: i32) -> Self {
        self.z_index = z;
        self
    }
}
