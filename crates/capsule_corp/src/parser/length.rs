use cssparser::{Parser, Token};

use crate::{
    CalcExpr, Dimension, Length, ParseResult,
    parser::error::{expected, integer_required, unexpected_token},
};

/// Parse a length: integer, integer + 'c', or percentage.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub fn parse_length<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Length> {
    if input
        .try_parse(|i| i.expect_function_matching("calc"))
        .is_ok()
    {
        return input.parse_nested_block(|i| {
            let expression = parse_calc_sum(i)?;

            Ok(Length::Calc(Box::new(expression)))
        });
    }

    let location = input.current_source_location();
    let token = input.next()?;

    match token {
        Token::Number {
            int_value: Some(n), ..
        } => Ok(Length::Cells(*n as u16)),

        Token::Number { .. } => integer_required(location),

        Token::Percentage { unit_value, .. } => Ok(Length::Percent(*unit_value * 100.0)),

        Token::Dimension {
            int_value: Some(n),
            unit,
            ..
        } if unit.eq_ignore_ascii_case("c") => Ok(Length::Cells(*n as u16)),

        Token::Dimension { unit, .. } if unit.eq_ignore_ascii_case("c") => {
            integer_required(location)
        }

        _ => unexpected_token(token, location),
    }
}

/// Parse a dimension: length, 'auto', or 'none'
pub fn parse_dimension<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Dimension> {
    if input.try_parse(|i| i.expect_ident_matching("auto")).is_ok() {
        return Ok(Dimension::Auto);
    }
    if input.try_parse(|i| i.expect_ident_matching("none")).is_ok() {
        return Ok(Dimension::None);
    }
    parse_length(input).map(Dimension::Length)
}

enum SumOp {
    Add,
    Sub,
}

/// Parse a calc sum: term (('+' | '-' ) term)*
fn parse_calc_sum<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, CalcExpr> {
    use SumOp::*;
    let mut left = parse_calc_product(input)?;

    loop {
        let op = input.try_parse(|i| {
            let token = i.next().map_err(|_| ())?;

            match &token {
                Token::Delim('+') => Ok(Add),
                Token::Delim('-') => Ok(Sub),
                _ => Err(()),
            }
        });

        match op {
            Ok(Add) => {
                let right = parse_calc_product(input)?;
                left = CalcExpr::Add(Box::new(left), Box::new(right));
            }
            Ok(Sub) => {
                let right = parse_calc_product(input)?;
                left = CalcExpr::Sub(Box::new(left), Box::new(right));
            }
            Err(()) => break,
        }
    }

    Ok(left)
}

enum ProductOp {
    Mult,
    Div,
}

/// Parse a calc prodcue: term(('*' | '/') number)*
fn parse_calc_product<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, CalcExpr> {
    use ProductOp::*;
    let mut left = parse_calc_factor(input)?;

    loop {
        let op = input.try_parse(|i| {
            let token = i.next().map_err(|_| ())?;

            match &token {
                Token::Delim('*') => Ok(Mult),
                Token::Delim('/') => Ok(Div),
                _ => Err(()),
            }
        });

        match op {
            Ok(Mult) => {
                let n = parse_number(input)?;
                left = CalcExpr::Mult(Box::new(left), n);
            }
            Ok(Div) => {
                let n = parse_number(input)?;
                left = CalcExpr::Div(Box::new(left), n);
            }
            Err(()) => break,
        }
    }

    Ok(left)
}

/// Parse a calc factor: '(' sum ')' | length
#[allow(clippy::cast_possible_truncation)]
fn parse_calc_factor<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, CalcExpr> {
    if input
        .try_parse(cssparser::Parser::expect_parenthesis_block)
        .is_ok()
    {
        return input.parse_nested_block(parse_calc_sum);
    }

    let location = input.current_source_location();
    let token = input.next()?.clone();

    match &token {
        Token::Number {
            int_value: Some(n), ..
        } => Ok(CalcExpr::Cells(*n as i16)),
        Token::Number { .. } => integer_required(location),
        Token::Percentage { unit_value, .. } => Ok(CalcExpr::Percent(*unit_value * 100.0)),
        Token::Dimension {
            int_value: Some(n),
            unit,
            ..
        } if unit.eq_ignore_ascii_case("c") => Ok(CalcExpr::Cells(*n as i16)),
        Token::Dimension { unit, .. } if unit.eq_ignore_ascii_case("c") => {
            integer_required(location)
        }
        _ => unexpected_token(&token, location),
    }
}

/// Parse a normal ass number (for calc)
fn parse_number<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, f32> {
    let location = input.current_source_location();
    let token = input.next()?;

    match token {
        Token::Number { value, .. } => Ok(*value),
        other => expected("number", other, location),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParseErrorKind;
    use cssparser::ParserInput;

    fn parse<'i, T>(
        s: &'i str,
        f: fn(&mut Parser<'i, '_>) -> ParseResult<'i, T>,
    ) -> ParseResult<'i, T> {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        f(&mut parser)
    }

    fn error_kind<T>(result: ParseResult<T>) -> Option<ParseErrorKind> {
        match result {
            Err(e) => match e.kind {
                cssparser::ParseErrorKind::Custom(k) => Some(k),
                cssparser::ParseErrorKind::Basic(_) => None,
            },
            Ok(_) => None,
        }
    }

    #[test]
    fn length_cells() {
        let l = parse("10", parse_length).expect("failed");
        assert_eq!(l, Length::Cells(10));
    }

    #[test]
    fn length_cells_unit() {
        let l = parse("10c", parse_length).expect("failed");
        assert_eq!(l, Length::Cells(10));
    }

    #[test]
    fn length_percent() {
        let l = parse("50%", parse_length).expect("failed");
        assert_eq!(l, Length::Percent(50.0));
    }

    #[test]
    fn length_float_rejected() {
        let result = parse("10.5", parse_length);
        assert_eq!(error_kind(result), Some(ParseErrorKind::IntegerRequired));
    }

    #[test]
    fn length_float_with_unit_rejected() {
        let result = parse("10.5c", parse_length);
        assert_eq!(error_kind(result), Some(ParseErrorKind::IntegerRequired));
    }

    #[test]
    fn dimension_auto() {
        let d = parse("auto", parse_dimension).expect("failed");
        assert_eq!(d, Dimension::Auto);
    }

    #[test]
    fn dimension_none() {
        let d = parse("none", parse_dimension).expect("failed");
        assert_eq!(d, Dimension::None);
    }

    #[test]
    fn dimension_length() {
        let d = parse("50%", parse_dimension).expect("failed");
        assert_eq!(d, Dimension::Length(Length::Percent(50.0)));
    }

    #[test]
    fn calc_simple_sub() {
        let l = parse("calc(100% - 10)", parse_length).expect("failed");
        let Length::Calc(expr) = l else {
            panic!("expected calc")
        };
        assert_eq!(expr.resolve(100), 90);
    }

    #[test]
    fn calc_multiply() {
        let l = parse("calc(50% * 2)", parse_length).expect("failed");
        let Length::Calc(expr) = l else {
            panic!("expected calc")
        };
        assert_eq!(expr.resolve(100), 100);
    }

    #[test]
    fn calc_divide() {
        let l = parse("calc(100 / 4)", parse_length).expect("failed");
        let Length::Calc(expr) = l else {
            panic!("expected calc")
        };
        assert_eq!(expr.resolve(0), 25);
    }

    #[test]
    fn calc_parens() {
        let l = parse("calc((100% - 20) / 2)", parse_length).expect("failed");
        let Length::Calc(expr) = l else {
            panic!("expected calc")
        };
        assert_eq!(expr.resolve(100), 40);
    }
}
