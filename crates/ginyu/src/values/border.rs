use crate::{Color, macros::keyword_enum};

keyword_enum! {

    #[derive(Default)]
    pub enum BorderStyle {
        #[default]
        None = "none",
        /// ─│┌┐└┘
        Solid = "solid",
        /// ┄┆┌┐└┘
        Dashed = "dashed",
        /// ···
        Dotted = "dotted",
        /// ═║╔╗╚╝
        Double = "double",
        /// ─│╭╮╰╯
        Rounded = "rounded",
    }
}

impl BorderStyle {
    #[must_use]
    pub const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Border {
    pub style: BorderStyle,
    pub color: Color,
}

impl Border {
    pub const NONE: Self = Self {
        style: BorderStyle::None,
        color: Color::Reset,
    };

    #[must_use]
    pub const fn new(style: BorderStyle, color: Color) -> Self {
        Self { style, color }
    }

    #[must_use]
    pub const fn solid(color: Color) -> Self {
        Self::new(BorderStyle::Solid, color)
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.style.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn border_style_from_name() {
        assert_eq!(BorderStyle::from_name("solid"), Some(BorderStyle::Solid));
        assert_eq!(
            BorderStyle::from_name("rounded"),
            Some(BorderStyle::Rounded)
        );
        assert_eq!(BorderStyle::from_name("thicc"), None);
    }

    #[test]
    fn border_style_is_none() {
        assert!(BorderStyle::None.is_none());
        assert!(!BorderStyle::Solid.is_none());
    }

    #[test]
    fn border_new() {
        let b = Border::new(BorderStyle::Double, Color::RED);
        assert_eq!(b.style, BorderStyle::Double);
        assert_eq!(b.color, Color::RED);
    }

    #[test]
    fn border_solid_helper() {
        let b = Border::solid(Color::CYAN);
        assert_eq!(b.style, BorderStyle::Solid);
        assert_eq!(b.color, Color::CYAN);
    }

    #[test]
    fn border_default() {
        let b = Border::default();
        assert!(b.is_none());
        assert_eq!(b.color, Color::Reset);
    }
}
