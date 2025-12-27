use cssparser::{Parser, parse_important};

use crate::{
    Color, Dimension, GlobalKeyword, Length, ParseErrorKind, ParseResult, Property, PropertyName,
    Shorthand, Specified, UnresolvedValue, Value,
    parser::{
        error::build_err, parse_border_style, parse_color, parse_dimension, parse_length,
        parse_number, parse_overflow, parse_value_with_vars, value::parse_property_value,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub property: Property,
    pub value: Specified<Value>,
    pub important: bool,
}

impl Declaration {
    pub fn new(property: Property, value: impl Into<Specified<Value>>) -> Self {
        Self {
            property,
            value: value.into(),
            important: false,
        }
    }

    pub const fn unresolved(property: Property, value: UnresolvedValue, important: bool) -> Self {
        Self {
            property,
            value: Specified::Unresolved(value),
            important,
        }
    }
}

fn try_parse_global(input: &mut Parser<'_, '_>) -> Option<Specified<Value>> {
    input
        .try_parse(|i| {
            let ident = i.expect_ident().map_err(|_| ())?;
            match GlobalKeyword::from_name(ident) {
                Some(GlobalKeyword::Inherit) => Ok(Specified::Inherit),
                Some(GlobalKeyword::Initial) => Ok(Specified::Initial),
                None => Err(()),
            }
        })
        .ok()
}

/// Parse a single declaration. Returns one or more declarations (if shorthands are expanded)
pub fn parse_declaration<'i>(
    name: &str,
    input: &mut Parser<'i, '_>,
) -> ParseResult<'i, Vec<Declaration>> {
    let location = input.current_source_location();

    let property_name = PropertyName::from_name(name)
        .ok_or_else(|| build_err(ParseErrorKind::UnknownProperty(name.to_string()), location))?;

    if let Some(global) = try_parse_global(input) {
        let important = parse_important(input).is_ok();
        return Ok(expand_to_properties(property_name, &global, important));
    }

    if let Some(unresolved) = parse_value_with_vars(input)? {
        let important = parse_important(input).is_ok();
        return Ok(expand_unresolved(property_name, &unresolved, important));
    }

    let mut declarations = match property_name {
        PropertyName::Longhand(property) => {
            let value = parse_property_value(property, input)?;
            vec![Declaration::new(property, value)]
        }
        PropertyName::Shorthand(shorthand) => parse_shorthand(shorthand, input)?,
    };

    let important = parse_important(input).is_ok();
    for decl in &mut declarations {
        decl.important = important;
    }

    Ok(declarations)
}

fn parse_shorthand<'i>(
    shorthand: Shorthand,
    input: &mut Parser<'i, '_>,
) -> ParseResult<'i, Vec<Declaration>> {
    use Shorthand::*;

    match shorthand {
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
    }
}

fn shorthand_properties(shorthand: Shorthand) -> Vec<Property> {
    use Shorthand::*;
    match shorthand {
        Background => vec![Property::BackgroundColor],
        Margin => vec![
            Property::MarginTop,
            Property::MarginRight,
            Property::MarginBottom,
            Property::MarginLeft,
        ],
        Padding => vec![
            Property::PaddingTop,
            Property::PaddingRight,
            Property::PaddingBottom,
            Property::PaddingLeft,
        ],
        Gap => vec![Property::RowGap, Property::ColumnGap],
        Overflow => vec![Property::OverflowX, Property::OverflowY],
        Flex => vec![
            Property::FlexGrow,
            Property::FlexShrink,
            Property::FlexBasis,
        ],
        Border => vec![
            Property::BorderTopStyle,
            Property::BorderRightStyle,
            Property::BorderBottomStyle,
            Property::BorderLeftStyle,
            Property::BorderTopColor,
            Property::BorderRightColor,
            Property::BorderBottomColor,
            Property::BorderLeftColor,
        ],
        BorderTop => vec![Property::BorderTopStyle, Property::BorderTopColor],
        BorderRight => vec![Property::BorderRightStyle, Property::BorderRightColor],
        BorderBottom => vec![Property::BorderBottomStyle, Property::BorderBottomColor],
        BorderLeft => vec![Property::BorderLeftStyle, Property::BorderLeftColor],
        BorderStyle => vec![
            Property::BorderTopStyle,
            Property::BorderRightStyle,
            Property::BorderBottomStyle,
            Property::BorderLeftStyle,
        ],
        BorderColor => vec![
            Property::BorderTopColor,
            Property::BorderRightColor,
            Property::BorderBottomColor,
            Property::BorderLeftColor,
        ],
    }
}

fn expand_to_properties(
    name: PropertyName,
    value: &Specified<Value>,
    important: bool,
) -> Vec<Declaration> {
    let properties = match name {
        PropertyName::Longhand(property) => vec![property],
        PropertyName::Shorthand(shorthand) => shorthand_properties(shorthand),
    };

    properties
        .into_iter()
        .map(|prop| Declaration {
            property: prop,
            value: value.clone(),
            important,
        })
        .collect()
}

fn expand_unresolved(
    name: PropertyName,
    unresolved: &UnresolvedValue,
    important: bool,
) -> Vec<Declaration> {
    let properties = match name {
        PropertyName::Longhand(property) => vec![property],
        PropertyName::Shorthand(shorthand) => shorthand_properties(shorthand),
    };

    properties
        .into_iter()
        .map(|prop| Declaration::unresolved(prop, unresolved.clone(), important))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use cssparser::ParserInput;

    fn parse(name: &str, value: &str) -> Result<Vec<Declaration>, String> {
        let mut input = ParserInput::new(value);
        let mut parser = Parser::new(&mut input);
        parse_declaration(name, &mut parser).map_err(|e| format!("{e:?}"))
    }

    #[test]
    fn simple_declaration() {
        let decls = parse("color", "red").expect("failed");
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].property, Property::Color);
        assert!(!decls[0].value.is_unresolved());
    }

    #[test]
    fn inherit_keyword() {
        let decls = parse("color", "inherit").expect("failed");
        assert_eq!(decls.len(), 1);
        assert_eq!(decls[0].value, Specified::Inherit);
    }

    #[test]
    fn initial_keyword() {
        let decls = parse("margin", "initial").expect("failed");
        assert_eq!(decls.len(), 4);
        assert!(decls.iter().all(|d| d.value == Specified::Initial));
    }

    #[test]
    fn var_function_deferred() {
        let decls = parse("color", "var(--primary)").expect("failed");
        assert_eq!(decls.len(), 1);
        assert!(decls[0].value.is_unresolved());
        assert_eq!(decls[0].value.as_unresolved_css(), Some("var(--primary)"));
    }

    #[test]
    fn var_in_shorthand() {
        let decls = parse("margin", "var(--spacing)").expect("failed");
        assert_eq!(decls.len(), 4);
        assert!(decls.iter().all(|d| d.value.is_unresolved()));
    }

    #[test]
    fn var_with_fallback() {
        let decls = parse("color", "var(--primary, blue)").expect("failed");
        assert!(decls[0].value.is_unresolved());

        let unresolved = decls[0].value.as_unresolved().expect("failed");
        assert_eq!(unresolved.references.len(), 1);
        assert!(unresolved.references[0].fallback.is_some());
    }

    #[test]
    fn important_flag() {
        let decls = parse("color", "red !important").expect("failed");
        assert!(decls[0].important);
    }

    #[test]
    fn important_with_var() {
        let decls = parse("color", "var(--x) !important").expect("failed");
        assert!(decls[0].important);
        assert!(decls[0].value.is_unresolved());
        // !important should NOT be in the css string
        assert!(
            !decls[0]
                .value
                .as_unresolved_css()
                .expect("failed")
                .contains("!important")
        );
    }

    #[test]
    fn nested_var_references_tracked() {
        let decls = parse("color", "var(--a, var(--b))").expect("failed");
        let unresolved = decls[0].value.as_unresolved().expect("failed");
        assert_eq!(unresolved.references.len(), 2);
    }
}
