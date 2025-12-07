pub use crate::{
    border::{BorderStyle, Borders},
    color::Color,
    text::{Alignment, Modifiers},
};

mod border;
mod color;
mod text;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Style {
    pub background: Color,
    pub borders: Borders,
    pub border_style: BorderStyle,
    pub border_color: Color,
    pub text_color: Color,
    pub text_alignment: Alignment,
    pub text_modifiers: Modifiers,
}

impl Style {
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
}
