use cssparser::{Parser, SourcePosition, Token, TokenSerializationType};
use ginyu_force::Pose;

use crate::{ParseErrorKind, ParseResult, UnresolvedValue, VarFallback, VarReference};

pub fn parse_value_with_vars<'i>(
    input: &mut Parser<'i, '_>,
) -> ParseResult<'i, Option<UnresolvedValue>> {
    let state = input.state();

    input.skip_whitespace();
    let start = input.position();

    let mut first_token_type = TokenSerializationType::Nothing;
    let mut last_token_type = TokenSerializationType::Nothing;
    let mut references = Vec::new();
    let mut has_var = false;

    collect_tokens_and_references(
        input,
        start,
        &mut first_token_type,
        &mut last_token_type,
        &mut references,
        &mut has_var,
    )?;

    if !has_var {
        input.reset(&state);
        return Ok(None);
    }

    let css = input.slice_from(start).trim().to_string();
    Ok(Some(UnresolvedValue {
        css,
        first_token_type,
        last_token_type,
        references,
    }))
}

fn collect_tokens_and_references<'i>(
    input: &mut Parser<'i, '_>,
    input_start: SourcePosition,
    first_token_type: &mut TokenSerializationType,
    last_token_type: &mut TokenSerializationType,
    references: &mut Vec<VarReference>,
    has_var: &mut bool,
) -> ParseResult<'i, ()> {
    collect_tokens_and_references_inner(
        input,
        input_start,
        first_token_type,
        last_token_type,
        references,
        has_var,
        true,
    )
}

fn collect_tokens_and_references_inner<'i>(
    input: &mut Parser<'i, '_>,
    input_start: SourcePosition,
    first_token_type: &mut TokenSerializationType,
    last_token_type: &mut TokenSerializationType,
    references: &mut Vec<VarReference>,
    has_var: &mut bool,
    check_important: bool,
) -> ParseResult<'i, ()> {
    let mut prev_token_type = TokenSerializationType::Nothing;

    loop {
        let token_start = input.position();
        let state_before_token = input.state();

        let token = match input.next_including_whitespace_and_comments() {
            Ok(t) => t.clone(),
            Err(_) => break,
        };

        if token == Token::Semicolon {
            input.reset(&state_before_token);
            break;
        }

        if check_important && token == Token::Delim('!') {
            let state_after_bang = input.state();
            if input
                .try_parse(|i| i.expect_ident_matching("important"))
                .is_ok()
            {
                input.reset(&state_before_token);
                break;
            }

            input.reset(&state_after_bang);
        }

        let token_type = token.serialization_type();

        first_token_type.set_if_nothing(token_type);
        *last_token_type = token_type;

        match token {
            Token::Function(name) if name.eq_ignore_ascii_case("var") => {
                *has_var = true;
                let var_start = byte_offset(input_start, token_start);
                let prev_type = prev_token_type;

                input.parse_nested_block(|i| {
                    parse_var_function(i, input_start, var_start, prev_type, references)
                })?;

                if let Some(last_ref) = references.last_mut() {
                    last_ref.end = byte_offset(input_start, input.position());

                    let state = input.state();
                    if let Ok(next) = input.next_including_whitespace_and_comments() {
                        last_ref.next_token_type = next.serialization_type();
                    }
                    input.reset(&state);
                }
            }
            Token::Function(_)
            | Token::ParenthesisBlock
            | Token::SquareBracketBlock
            | Token::CurlyBracketBlock => {
                input.parse_nested_block(|i| {
                    let mut nested_first = TokenSerializationType::Nothing;
                    let mut nested_last = TokenSerializationType::Nothing;

                    collect_tokens_and_references_inner(
                        i,
                        input_start,
                        &mut nested_first,
                        &mut nested_last,
                        references,
                        has_var,
                        false,
                    )
                })?;
            }
            _ => {}
        }

        prev_token_type = token_type;
    }

    Ok(())
}

fn parse_var_function<'i>(
    input: &mut Parser<'i, '_>,
    input_start: SourcePosition,
    start: usize,
    prev_token_type: TokenSerializationType,
    references: &mut Vec<VarReference>,
) -> ParseResult<'i, ()> {
    let name_token = input.expect_ident()?;

    let name = if let Some(stripped) = name_token.strip_prefix("--") {
        Pose::from(stripped)
    } else {
        return Err(input.new_custom_error(ParseErrorKind::InvalidVariable));
    };

    let fallback = parse_var_fallback(input, input_start, references)?;

    references.push(VarReference {
        name,
        start,
        end: 0,
        prev_token_type,
        next_token_type: TokenSerializationType::Nothing,
        fallback,
    });

    Ok(())
}

fn parse_var_fallback<'i>(
    input: &mut Parser<'i, '_>,
    input_start: SourcePosition,
    references: &mut Vec<VarReference>,
) -> ParseResult<'i, Option<VarFallback>> {
    if input.try_parse(cssparser::Parser::expect_comma).is_err() {
        return Ok(None);
    }

    input.skip_whitespace();

    let start = byte_offset(input_start, input.position());

    let mut first_token_type = TokenSerializationType::Nothing;
    let mut last_token_type = TokenSerializationType::Nothing;
    let mut nested_refs = Vec::new();
    let mut nested_has_vars = false;

    collect_tokens_and_references_inner(
        input,
        input_start,
        &mut first_token_type,
        &mut last_token_type,
        &mut nested_refs,
        &mut nested_has_vars,
        false,
    )?;

    references.extend(nested_refs);

    Ok(Some(VarFallback {
        start,
        first_token_type,
        last_token_type,
    }))
}

fn byte_offset(start: SourcePosition, current: SourcePosition) -> usize {
    current.byte_index() - start.byte_index()
}

#[cfg(test)]
mod tests {
    use ginyu_force::Pose;
    use rustc_hash::FxHashMap;

    use crate::SubstituteError;

    use super::*;
    use cssparser::ParserInput;

    fn parse(s: &str) -> Option<UnresolvedValue> {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        parse_value_with_vars(&mut parser).ok().flatten()
    }

    fn make_vars(pairs: &[(&str, &str)]) -> FxHashMap<Pose, String> {
        pairs
            .iter()
            .map(|(k, v)| (Pose::from(*k), (*v).to_string()))
            .collect()
    }

    #[test]
    fn no_var_returns_none() {
        assert!(parse("red").is_none());
        assert!(parse("10 solid blue").is_none());
        assert!(parse("calc(100% - 10)").is_none());
    }

    #[test]
    fn simple_var() {
        let result = parse("var(--primary)").expect("failed");
        assert_eq!(result.references.len(), 1);
        assert!(result.references[0].name == "primary");
        assert!(result.references[0].fallback.is_none());
    }

    #[test]
    fn var_with_fallback() {
        let result = parse("var(--primary, blue)").expect("failed");
        assert_eq!(result.references.len(), 1);
        assert!(result.references[0].name == "primary");
        assert!(result.references[0].fallback.is_some());
    }

    #[test]
    fn nested_var_in_function() {
        let result = parse("calc(var(--size) + 10)").expect("failed");
        assert_eq!(result.references.len(), 1);
        assert!(result.references[0].name == "size");
    }

    #[test]
    fn multiple_vars() {
        let result = parse("var(--a) var(--b)").expect("failed");
        assert_eq!(result.references.len(), 2);
        assert!(result.references[0].name == "a");
        assert!(result.references[1].name == "b");
    }

    #[test]
    fn nested_var_in_fallback() {
        let result = parse("var(--a, var(--b))").expect("failed");
        assert_eq!(result.references.len(), 2);
        // Inner var is collected first during fallback parsing
        assert!(result.references[0].name == "b");
        assert!(result.references[1].name == "a");
    }

    #[test]
    fn token_types_are_tracked() {
        let result = parse("1 + var(--x)").expect("failed");
        assert!(result.first_token_type != TokenSerializationType::Nothing);
        assert!(result.last_token_type != TokenSerializationType::Nothing);
    }

    #[test]
    fn byte_offsets_correct() {
        let result = parse("var(--foo)").expect("failed");
        assert_eq!(result.references[0].start, 0);
        assert_eq!(result.references[0].end, 10);

        let result = parse("calc(var(--foo) + 1)").expect("failed");
        assert_eq!(result.references[0].start, 5);
        assert_eq!(result.references[0].end, 15);
    }

    #[test]
    fn substitute_simple() {
        let unresolved = parse("var(--color)").expect("failed");
        let vars = make_vars(&[("color", "red")]);

        let result = unresolved.substitute(|name| vars.get(&name).map(String::as_str));
        assert_eq!(result.expect("failed"), "red");
    }

    #[test]
    fn substitute_with_surrounding_text() {
        let unresolved = parse("1px solid var(--color)").expect("failed");
        let vars = make_vars(&[("color", "blue")]);

        let result = unresolved.substitute(|name| vars.get(&name).map(String::as_str));
        assert_eq!(result.expect("failed"), "1px solid blue");
    }

    #[test]
    fn substitute_multiple_vars() {
        let unresolved = parse("var(--x) var(--y)").expect("failed");
        let vars = make_vars(&[("x", "10"), ("y", "20")]);

        let result = unresolved.substitute(|name| vars.get(&name).map(String::as_str));
        assert_eq!(result.expect("failed"), "10 20");
    }

    #[test]
    fn substitute_uses_fallback() {
        let unresolved = parse("var(--missing, fallback-value)").expect("failed");
        let vars: FxHashMap<Pose, String> = FxHashMap::default();

        let result = unresolved.substitute(|name| vars.get(&name).map(String::as_str));
        assert_eq!(result.expect("failed"), "fallback-value");
    }

    #[test]
    fn substitute_prefers_value_over_fallback() {
        let unresolved = parse("var(--color, fallback)").expect("failed");
        let vars = make_vars(&[("color", "red")]);

        let result = unresolved.substitute(|name| vars.get(&name).map(String::as_str));
        assert_eq!(result.expect("failed"), "red");
    }

    #[test]
    fn substitute_undefined_no_fallback_errors() {
        let unresolved = parse("var(--missing)").expect("failed");
        let vars: FxHashMap<Pose, String> = FxHashMap::default();

        let result = unresolved.substitute(|name| vars.get(&name).map(String::as_str));
        assert!(
            matches!(result, Err(SubstituteError::UndefinedVariable(name)) if name == "missing")
        );
    }

    #[test]
    fn substitute_in_calc() {
        let unresolved = parse("calc(var(--size) + 10)").expect("failed");
        let vars = make_vars(&[("size", "100")]);

        let result = unresolved.substitute(|name| vars.get(&name).map(String::as_str));
        assert_eq!(result.expect("failed"), "calc(100 + 10)");
    }

    #[test]
    fn substitute_empty_value() {
        let unresolved = parse("var(--empty)").expect("failed");
        let vars = make_vars(&[("empty", "")]);

        let result = unresolved.substitute(|name| vars.get(&name).map(String::as_str));
        assert_eq!(result.expect("failed"), "");
    }
}
