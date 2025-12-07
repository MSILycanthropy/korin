use crate::{
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
