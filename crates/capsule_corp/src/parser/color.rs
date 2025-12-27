use cssparser::{Parser, SourceLocation, Token};

use crate::{
    BasicColor, Color, ParseErrorKind, ParseResult,
    parser::error::{error, expected},
};

/// Parse a color value
pub fn parse_color<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Color> {
    let location = input.current_source_location();
    let token = input.next()?.clone();

    match token {
        Token::Function(name) => input.parse_nested_block(|i| parse_color_function(&name, i)),
        Token::Hash(hex) | Token::IDHash(hex) => parse_hex_color(&hex, location),
        Token::Ident(name) => parse_named_color(&name, location),
        other => expected("color", &other, location),
    }
}

fn parse_color_function<'i>(name: &str, input: &mut Parser<'i, '_>) -> ParseResult<'i, Color> {
    let location = input.current_source_location();

    match name {
        "rgb" => {
            dbg!("parsing RGB");
            let r = parse_u8(input)?;
            input.expect_comma()?;
            let g = parse_u8(input)?;
            input.expect_comma()?;
            let b = parse_u8(input)?;

            Ok(Color::Rgb(r, g, b))
        }
        "ansi" => {
            let n = parse_u8(input)?;
            Ok(Color::Ansi(n))
        }
        _ => error(ParseErrorKind::UnknownFunction(name.to_string()), location),
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn parse_u8<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, u8> {
    let location = input.current_source_location();
    let token = input.next()?;

    match &token {
        Token::Number {
            int_value: Some(n), ..
        } => {
            if *n >= 0 && *n <= 255 {
                Ok(*n as u8)
            } else {
                error(
                    ParseErrorKind::OutOfRange {
                        value: i64::from(*n),
                        min: 0,
                        max: 255,
                    },
                    location,
                )
            }
        }
        _ => expected("integer 0-255", token, location),
    }
}

fn parse_hex_color<'i>(hex: &str, location: SourceLocation) -> ParseResult<'i, Color> {
    let hex = hex.trim_start_matches('#');

    match hex.len() {
        3 => {
            let r = parse_hex_digit(&hex[0..1], location)?;
            let g = parse_hex_digit(&hex[1..2], location)?;
            let b = parse_hex_digit(&hex[2..3], location)?;

            Ok(Color::Rgb(r * 17, g * 17, b * 17))
        }
        6 => {
            let r = parse_hex_digit(&hex[0..2], location)?;
            let g = parse_hex_digit(&hex[2..4], location)?;
            let b = parse_hex_digit(&hex[4..6], location)?;

            Ok(Color::Rgb(r, g, b))
        }

        _ => error(ParseErrorKind::InvalidHexColor, location),
    }
}

fn parse_hex_digit<'i>(digit: &str, location: SourceLocation) -> ParseResult<'i, u8> {
    u8::from_str_radix(digit, 16).or_else(|_| error(ParseErrorKind::InvalidHexColor, location))
}

fn parse_named_color<'i>(name: &str, location: SourceLocation) -> ParseResult<'i, Color> {
    if name.eq_ignore_ascii_case("reset") {
        return Ok(Color::Reset);
    }

    if let Some(color) = name.strip_prefix("bright-").and_then(BasicColor::from_name) {
        return Ok(Color::Bright(color));
    }

    if let Some(color) = BasicColor::from_name(name) {
        return Ok(Color::Basic(color));
    }

    error(ParseErrorKind::UnknownColor(name.to_string()), location)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cssparser::ParserInput;

    fn parse(s: &str) -> ParseResult<'_, Color> {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        parse_color(&mut parser)
    }

    #[test]
    fn basic_colors() {
        assert_eq!(
            parse("black").expect("failed"),
            Color::Basic(BasicColor::Black)
        );
        assert_eq!(parse("red").expect("failed"), Color::Basic(BasicColor::Red));
        assert_eq!(
            parse("cyan").expect("failed"),
            Color::Basic(BasicColor::Cyan)
        );
    }

    #[test]
    fn bright_colors() {
        assert_eq!(
            parse("bright-red").expect("failed"),
            Color::Bright(BasicColor::Red)
        );
        assert_eq!(
            parse("bright-cyan").expect("failed"),
            Color::Bright(BasicColor::Cyan)
        );
    }

    #[test]
    fn reset() {
        assert_eq!(parse("reset").expect("failed"), Color::Reset);
    }

    #[test]
    fn ansi() {
        assert_eq!(parse("ansi(196)").expect("failed"), Color::Ansi(196));
        assert_eq!(parse("ansi(0)").expect("failed"), Color::Ansi(0));
        assert_eq!(parse("ansi(255)").expect("failed"), Color::Ansi(255));
    }

    #[test]
    fn rgb() {
        assert_eq!(
            parse("rgb(255, 100, 50)").expect("failed"),
            Color::Rgb(255, 100, 50)
        );
        assert_eq!(parse("rgb(0, 0, 0)").expect("failed"), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn hex_short() {
        assert_eq!(parse("#f00").expect("failed"), Color::Rgb(255, 0, 0));
        assert_eq!(parse("#0f0").expect("failed"), Color::Rgb(0, 255, 0));
        assert_eq!(parse("#abc").expect("failed"), Color::Rgb(170, 187, 204));
    }

    #[test]
    fn hex_long() {
        assert_eq!(parse("#ff0000").expect("failed"), Color::Rgb(255, 0, 0));
        assert_eq!(parse("#00ff00").expect("failed"), Color::Rgb(0, 255, 0));
        assert_eq!(parse("#aabbcc").expect("failed"), Color::Rgb(170, 187, 204));
    }

    #[test]
    fn unknown_color() {
        assert!(parse("purple").is_err());
        assert!(parse("bright-purple").is_err());
    }

    #[test]
    fn ansi_out_of_range() {
        assert!(parse("ansi(256)").is_err());
        assert!(parse("ansi(-1)").is_err());
    }

    #[test]
    fn invalid_hex() {
        assert!(parse("#ff").is_err());
        assert!(parse("#fffffff").is_err());
    }
}
