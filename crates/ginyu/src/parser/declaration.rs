use cssparser::{Parser, parse_important};

use crate::{
    Color, Dimension, Length, ParseErrorKind, ParseResult, Property, PropertyName, Shorthand,
    Value,
    parser::{
        error::build_err, parse_border_style, parse_color, parse_dimension, parse_length,
        parse_number, parse_overflow, value::parse_property_value,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: Property,
    pub value: Value,
    pub important: bool,
}

impl Declaration {
    pub const fn new(property: Property, value: Value) -> Self {
        Self {
            property,
            value,
            important: false,
        }
    }
}

/// Parse a single declaration. Returns one or more declarations (if shorthands are expanded)
pub fn parse_declaration<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Vec<Declaration>> {
    let location = input.current_source_location();

    let name = input.expect_ident()?;
    let property_name = PropertyName::from_name(name)
        .ok_or_else(|| build_err(ParseErrorKind::UnknownProperty(name.to_string()), location))?;

    input.expect_colon()?;

    match property_name {
        PropertyName::Longhand(property) => {
            let value = parse_property_value(property, input)?;
            let important = parse_important(input).is_ok();

            Ok(vec![Declaration {
                property,
                value,
                important,
            }])
        }
        PropertyName::Shorthand(shorthand) => parse_shorthand(shorthand, input),
    }
}

fn parse_shorthand<'i>(
    shorthand: Shorthand,
    input: &mut Parser<'i, '_>,
) -> ParseResult<'i, Vec<Declaration>> {
    use Shorthand::*;

    let declarations = match shorthand {
        Margin => parse_box_shorthand(
            input,
            [
                Property::MarginTop,
                Property::MarginRight,
                Property::MarginBottom,
                Property::MarginLeft,
            ],
        ),
        Padding => parse_box_shorthand(
            input,
            [
                Property::PaddingTop,
                Property::PaddingRight,
                Property::PaddingBottom,
                Property::PaddingLeft,
            ],
        ),
        Gap => parse_gap_shorthand(input),
        Overflow => parse_overflow_shorthand(input),
        Flex => parse_flex_shorthand(input),
        Border => parse_border_shorthand(input),
        BorderStyle => parse_border_style_shorthand(input),
        BorderColor => parse_border_color_shorthand(input),
        BorderTop => {
            parse_border_side_shorthand(input, Property::BorderTopStyle, Property::BorderTopColor)
        }
        BorderRight => parse_border_side_shorthand(
            input,
            Property::BorderRightStyle,
            Property::BorderRightColor,
        ),
        BorderBottom => parse_border_side_shorthand(
            input,
            Property::BorderBottomStyle,
            Property::BorderBottomColor,
        ),
        BorderLeft => {
            parse_border_side_shorthand(input, Property::BorderLeftStyle, Property::BorderLeftColor)
        }
        Background => parse_background_shorthand(input),
    }?;

    let important = parse_important(input).is_ok();

    Ok(declarations
        .into_iter()
        .map(|mut d| {
            d.important = important;
            d
        })
        .collect())
}

fn parse_box_shorthand<'i>(
    input: &mut Parser<'i, '_>,
    properties: [Property; 4],
) -> ParseResult<'i, Vec<Declaration>> {
    let mut values = Vec::with_capacity(4);

    values.push(parse_length(input)?);

    for _ in 0..3 {
        if let Ok(value) = input.try_parse(parse_length) {
            values.push(value);
        } else {
            break;
        }
    }

    let (top, right, bottom, left) = match values.len() {
        1 => (
            values[0].clone(),
            values[0].clone(),
            values[0].clone(),
            values[0].clone(),
        ),
        2 => (
            values[0].clone(),
            values[1].clone(),
            values[0].clone(),
            values[1].clone(),
        ),
        3 => (
            values[0].clone(),
            values[1].clone(),
            values[2].clone(),
            values[1].clone(),
        ),
        4 => (
            values[0].clone(),
            values[1].clone(),
            values[2].clone(),
            values[3].clone(),
        ),
        _ => unreachable!(),
    };

    Ok(vec![
        Declaration::new(properties[0], Value::Length(top)),
        Declaration::new(properties[1], Value::Length(right)),
        Declaration::new(properties[2], Value::Length(bottom)),
        Declaration::new(properties[3], Value::Length(left)),
    ])
}

fn parse_gap_shorthand<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Vec<Declaration>> {
    let row = parse_length(input)?;
    let column = input
        .try_parse(parse_length)
        .unwrap_or_else(|_| row.clone());

    Ok(vec![
        Declaration::new(Property::RowGap, Value::Length(row)),
        Declaration::new(Property::RowGap, Value::Length(column)),
    ])
}

fn parse_overflow_shorthand<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Vec<Declaration>> {
    let x = parse_overflow(input)?;
    let y = input.try_parse(parse_overflow).unwrap_or(x);

    Ok(vec![
        Declaration::new(Property::OverflowX, Value::Overflow(x)),
        Declaration::new(Property::OverflowY, Value::Overflow(y)),
    ])
}

fn parse_flex_shorthand<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Vec<Declaration>> {
    if input.try_parse(|i| i.expect_ident_matching("none")).is_ok() {
        return Ok(vec![
            Declaration::new(Property::FlexGrow, Value::Number(0.0)),
            Declaration::new(Property::FlexShrink, Value::Number(0.0)),
            Declaration::new(Property::FlexBasis, Value::Number(0.0)),
        ]);
    }

    if input.try_parse(|i| i.expect_ident_matching("auto")).is_ok() {
        return Ok(vec![
            Declaration::new(Property::FlexGrow, Value::Number(1.0)),
            Declaration::new(Property::FlexShrink, Value::Number(1.0)),
            Declaration::new(Property::FlexBasis, Value::Dimension(Dimension::Auto)),
        ]);
    }

    // Parse <grow> <shrink>? <basis>?
    let grow = parse_number(input)?;
    let shrink = input.try_parse(parse_number).unwrap_or(1.0);
    let basis = input
        .try_parse(parse_dimension)
        .unwrap_or(Dimension::Length(Length::Cells(0)));

    Ok(vec![
        Declaration::new(Property::FlexGrow, Value::Number(grow)),
        Declaration::new(Property::FlexShrink, Value::Number(shrink)),
        Declaration::new(Property::FlexBasis, Value::Dimension(basis)),
    ])
}

/// Parse <style> <color>?
fn parse_border_shorthand<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Vec<Declaration>> {
    let style = parse_border_style(input)?;
    let color = input.try_parse(parse_color).unwrap_or(Color::Reset);

    Ok(vec![
        Declaration::new(Property::BorderTopStyle, Value::BorderStyle(style)),
        Declaration::new(Property::BorderRightStyle, Value::BorderStyle(style)),
        Declaration::new(Property::BorderBottomStyle, Value::BorderStyle(style)),
        Declaration::new(Property::BorderLeftStyle, Value::BorderStyle(style)),
        Declaration::new(Property::BorderTopColor, Value::Color(color)),
        Declaration::new(Property::BorderRightColor, Value::Color(color)),
        Declaration::new(Property::BorderBottomColor, Value::Color(color)),
        Declaration::new(Property::BorderLeftColor, Value::Color(color)),
    ])
}

fn parse_border_style_shorthand<'i>(
    input: &mut Parser<'i, '_>,
) -> ParseResult<'i, Vec<Declaration>> {
    use crate::parser::parse_border_style;

    let mut values = Vec::with_capacity(4);
    values.push(parse_border_style(input)?);

    for _ in 0..3 {
        if let Ok(v) = input.try_parse(parse_border_style) {
            values.push(v);
        } else {
            break;
        }
    }

    let (top, right, bottom, left) = match values.len() {
        1 => (values[0], values[0], values[0], values[0]),
        2 => (values[0], values[1], values[0], values[1]),
        3 => (values[0], values[1], values[2], values[1]),
        4 => (values[0], values[1], values[2], values[3]),
        _ => unreachable!(),
    };

    Ok(vec![
        Declaration::new(Property::BorderTopStyle, Value::BorderStyle(top)),
        Declaration::new(Property::BorderRightStyle, Value::BorderStyle(right)),
        Declaration::new(Property::BorderBottomStyle, Value::BorderStyle(bottom)),
        Declaration::new(Property::BorderLeftStyle, Value::BorderStyle(left)),
    ])
}

fn parse_border_color_shorthand<'i>(
    input: &mut Parser<'i, '_>,
) -> ParseResult<'i, Vec<Declaration>> {
    use crate::parser::parse_color;

    let mut values = Vec::with_capacity(4);
    values.push(parse_color(input)?);

    for _ in 0..3 {
        if let Ok(v) = input.try_parse(parse_color) {
            values.push(v);
        } else {
            break;
        }
    }

    let (top, right, bottom, left) = match values.len() {
        1 => (values[0], values[0], values[0], values[0]),
        2 => (values[0], values[1], values[0], values[1]),
        3 => (values[0], values[1], values[2], values[1]),
        4 => (values[0], values[1], values[2], values[3]),
        _ => unreachable!(),
    };

    Ok(vec![
        Declaration::new(Property::BorderTopColor, Value::Color(top)),
        Declaration::new(Property::BorderRightColor, Value::Color(right)),
        Declaration::new(Property::BorderBottomColor, Value::Color(bottom)),
        Declaration::new(Property::BorderLeftColor, Value::Color(left)),
    ])
}

/// Parse border-<side> shorthand: <style> <color>?
fn parse_border_side_shorthand<'i>(
    input: &mut Parser<'i, '_>,
    style_prop: Property,
    color_prop: Property,
) -> ParseResult<'i, Vec<Declaration>> {
    use crate::parser::{parse_border_style, parse_color};
    use crate::values::Color;

    let style = parse_border_style(input)?;
    let color = input.try_parse(parse_color).unwrap_or(Color::Reset);

    Ok(vec![
        Declaration::new(style_prop, Value::BorderStyle(style)),
        Declaration::new(color_prop, Value::Color(color)),
    ])
}

fn parse_background_shorthand<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Vec<Declaration>> {
    use crate::parser::parse_color;

    let color = parse_color(input)?;

    Ok(vec![Declaration::new(
        Property::BackgroundColor,
        Value::Color(color),
    )])
}
