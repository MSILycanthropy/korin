use cssparser::{Parser, Token};

use crate::{
    AlignItems, AlignSelf, BorderStyle, Display, FlexDirection, FlexWrap, FontStyle, FontWeight,
    JustifyContent, Overflow, OverflowWrap, ParseErrorKind, ParseResult, TextAlign, TextDecoration,
    VerticalAlign, Visibility, WhiteSpace,
    parser::error::{build_err, expected},
};

/// Parse a keyword using a `from_name` function.
///
/// Usage:
/// ```ignore
/// let display = parse_keyword(input, Display::from_name, "display")?;
/// ```
fn parse_keyword<'i, T>(
    input: &mut Parser<'i, '_>,
    from_name: fn(&str) -> Option<T>,
    property: &'static str,
) -> ParseResult<'i, T> {
    let location = input.current_source_location();
    let token = input.next()?;

    match token {
        Token::Ident(name) => from_name(name).ok_or_else(|| {
            build_err(
                ParseErrorKind::UnknownKeyword {
                    keyword: name.to_string(),
                    property,
                },
                location,
            )
        }),
        other => expected("keyword", other, location),
    }
}

/// Generate parse functions for keyword enums.
macro_rules! keyword_parsers {
    ($($fn_name:ident => $type:ty, $property:literal);* $(;)?) => {
        $(
            pub fn $fn_name<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, $type> {
                parse_keyword(input, <$type>::from_name, $property)
            }
        )*
    };
}

keyword_parsers! {
    parse_display => Display, "display";
    parse_flex_direction => FlexDirection, "flex-direction";
    parse_flex_wrap => FlexWrap, "flex-wrap";
    parse_justify_content => JustifyContent, "justify-content";
    parse_align_items => AlignItems, "align-items";
    parse_align_self => AlignSelf, "align-self";

    parse_text_align => TextAlign, "text-align";
    parse_vertical_align => VerticalAlign, "vertical-align";
    parse_font_weight => FontWeight, "font-weight";
    parse_font_style => FontStyle, "font-style";
    parse_text_decoration => TextDecoration, "text-decoration";
    parse_white_space => WhiteSpace, "white-space";
    parse_overflow_wrap => OverflowWrap, "overflow-wrap";

    parse_overflow => Overflow, "overflow";
    parse_visibility => Visibility, "visibility";

    parse_border_style => BorderStyle, "border-style";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::*;
    use cssparser::ParserInput;

    fn parse<'i, T>(
        s: &'i str,
        f: fn(&mut Parser<'i, '_>) -> ParseResult<'i, T>,
    ) -> Result<T, String> {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        f(&mut parser).map_err(|e| format!("{:?}", e.kind))
    }

    #[test]
    fn display() {
        assert_eq!(
            parse("block", parse_display).expect("failed"),
            Display::Block
        );
        assert_eq!(parse("flex", parse_display).expect("failed"), Display::Flex);
        assert_eq!(parse("none", parse_display).expect("failed"), Display::None);
    }

    #[test]
    fn display_unknown() {
        let err = parse("banana", parse_display).expect_err("failed");
        assert!(err.contains("UnknownKeyword"));
    }

    #[test]
    fn flex_direction() {
        assert_eq!(
            parse("row", parse_flex_direction).expect("failed"),
            FlexDirection::Row
        );
        assert_eq!(
            parse("column", parse_flex_direction).expect("failed"),
            FlexDirection::Column
        );
        assert_eq!(
            parse("row-reverse", parse_flex_direction).expect("failed"),
            FlexDirection::RowReverse
        );
    }

    #[test]
    fn text_align() {
        assert_eq!(
            parse("left", parse_text_align).expect("failed"),
            TextAlign::Left
        );
        assert_eq!(
            parse("center", parse_text_align).expect("failed"),
            TextAlign::Center
        );
        assert_eq!(
            parse("right", parse_text_align).expect("failed"),
            TextAlign::Right
        );
    }

    #[test]
    fn border_style() {
        assert_eq!(
            parse("none", parse_border_style).expect("failed"),
            BorderStyle::None
        );
        assert_eq!(
            parse("solid", parse_border_style).expect("failed"),
            BorderStyle::Solid
        );
        assert_eq!(
            parse("rounded", parse_border_style).expect("failed"),
            BorderStyle::Rounded
        );
    }
}
