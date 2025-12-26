use cssparser::Parser;

use crate::{
    ParseResult, Property, Value,
    parser::{
        parse_align_items, parse_align_self, parse_border_style, parse_color, parse_dimension,
        parse_display, parse_flex_direction, parse_flex_wrap, parse_font_style, parse_font_weight,
        parse_integer, parse_justify_content, parse_length, parse_number, parse_overflow,
        parse_overflow_wrap, parse_text_align, parse_text_decoration, parse_vertical_align,
        parse_visibility, parse_white_space,
    },
};

/// Parse a value for a given property
pub fn parse_property_value<'i>(
    property: Property,
    input: &mut Parser<'i, '_>,
) -> ParseResult<'i, Value> {
    use Property::*;

    match property {
        Display => parse_display(input).map(Value::Display),
        FlexDirection => parse_flex_direction(input).map(Value::FlexDirection),
        FlexWrap => parse_flex_wrap(input).map(Value::FlexWrap),
        JustifyContent => parse_justify_content(input).map(Value::JustifyContent),
        AlignItems => parse_align_items(input).map(Value::AlignItems),

        FlexGrow | FlexShrink => parse_number(input).map(Value::Number),
        FlexBasis => parse_dimension(input).map(Value::Dimension),
        AlignSelf => parse_align_self(input).map(Value::AlignSelf),

        // TODO: Hmm.. parse grid right
        GridTemplateColumns | GridTemplateRows | GridColumn | GridRow | Width | Height
        | MinWidth | MinHeight | MaxWidth | MaxHeight => {
            parse_dimension(input).map(Value::Dimension)
        }

        RowGap | ColumnGap | MarginTop | MarginBottom | MarginLeft | MarginRight | PaddingTop
        | PaddingBottom | PaddingLeft | PaddingRight => parse_length(input).map(Value::Length),

        BorderTopStyle | BorderBottomStyle | BorderLeftStyle | BorderRightStyle => {
            parse_border_style(input).map(Value::BorderStyle)
        }

        BorderTopColor | BorderBottomColor | BorderLeftColor | BorderRightColor => {
            parse_color(input).map(Value::Color)
        }

        Color | BackgroundColor => parse_color(input).map(Value::Color),

        FontWeight => parse_font_weight(input).map(Value::FontWeight),
        FontStyle => parse_font_style(input).map(Value::FontStyle),
        TextDecoration => parse_text_decoration(input).map(Value::TextDecoration),
        TextAlign => parse_text_align(input).map(Value::TextAlign),
        VerticalAlign => parse_vertical_align(input).map(Value::VerticalAlign),
        WhiteSpace => parse_white_space(input).map(Value::WhiteSpace),
        OverflowWrap => parse_overflow_wrap(input).map(Value::OverflowWrap),

        OverflowX | OverflowY => parse_overflow(input).map(Value::Overflow),
        Visibility => parse_visibility(input).map(Value::Visibility),

        ZIndex => parse_integer(input).map(Value::Integer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::values::*;
    use cssparser::ParserInput;

    fn parse(property: Property, s: &str) -> Result<Value, String> {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        parse_property_value(property, &mut parser).map_err(|e| format!("{:?}", e.kind))
    }

    #[test]
    fn display_property() {
        let v = parse(Property::Display, "flex").expect("failed");
        assert_eq!(v.as_display(), Some(&Display::Flex));
    }

    #[test]
    fn width_property() {
        let v = parse(Property::Width, "100").expect("failed");
        assert_eq!(
            v.as_dimension(),
            Some(&Dimension::Length(Length::Cells(100)))
        );

        let v = parse(Property::Width, "auto").expect("failed");
        assert_eq!(v.as_dimension(), Some(&Dimension::Auto));

        let v = parse(Property::Width, "50%").expect("failed");
        assert_eq!(
            v.as_dimension(),
            Some(&Dimension::Length(Length::Percent(50.0)))
        );
    }

    #[test]
    fn margin_property() {
        let v = parse(Property::MarginTop, "10").expect("failed");
        assert_eq!(v.as_length(), Some(&Length::Cells(10)));

        let v = parse(Property::MarginLeft, "-5").expect("failed");
        assert_eq!(v.as_length(), Some(&Length::Cells(-5)));
    }

    #[test]
    fn color_property() {
        let v = parse(Property::Color, "red").expect("failed");
        assert_eq!(v.as_color(), Some(&Color::RED));

        let v = parse(Property::BackgroundColor, "#ff0000").expect("failed");
        assert_eq!(v.as_color(), Some(&Color::Rgb(255, 0, 0)));
    }

    #[test]
    fn flex_grow_property() {
        let v = parse(Property::FlexGrow, "2.5").expect("failed");
        assert_eq!(v.as_number(), Some(2.5));
    }

    #[test]
    fn z_index_property() {
        let v = parse(Property::ZIndex, "10").expect("failed");
        assert_eq!(v.as_integer(), Some(10));

        let v = parse(Property::ZIndex, "-1").expect("failed");
        assert_eq!(v.as_integer(), Some(-1));
    }

    #[test]
    fn border_style_property() {
        let v = parse(Property::BorderTopStyle, "solid").expect("failed");
        assert_eq!(v.as_border_style(), Some(&BorderStyle::Solid));
    }

    #[test]
    fn border_color_property() {
        let v = parse(Property::BorderTopColor, "cyan").expect("failed");
        assert_eq!(v.as_color(), Some(&Color::CYAN));
    }
}
