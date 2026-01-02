mod color;
mod declaration;
mod error;
mod keyword;
mod length;
mod rule;
mod selector;
mod stylesheet;
mod unresolved;
mod value;

use color::parse_color;
use cssparser::{Parser, Token};
pub use declaration::{Declaration, parse_inline_style};
use keyword::{
    parse_align_items, parse_align_self, parse_border_style, parse_display, parse_flex_direction,
    parse_flex_wrap, parse_font_style, parse_font_weight, parse_justify_content, parse_overflow,
    parse_overflow_wrap, parse_text_align, parse_text_decoration, parse_vertical_align,
    parse_visibility, parse_white_space,
};
use length::{parse_dimension, parse_length};
pub use rule::Rule;
pub use selector::parse_selector;
pub use unresolved::parse_value_with_vars;
pub use value::parse_property_value;

pub use error::{ParseErrorKind, ParseResult};
pub use stylesheet::*;

use crate::parser::error::expected;

fn parse_number<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, f32> {
    let location = input.current_source_location();
    let token = input.next()?;

    match token {
        Token::Number { value, .. } => Ok(*value),
        other => expected("number", other, location),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn parse_integer<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, i16> {
    let location = input.current_source_location();
    let token = input.next()?;

    match token {
        Token::Number {
            int_value: Some(n), ..
        } => Ok(*n as i16),
        other => expected("integer", other, location),
    }
}
