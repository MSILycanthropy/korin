use cssparser::{Parser, Token};

use crate::{
    ParseResult,
    parser::declaration::{Declaration, parse_declaration},
};

#[derive(Clone, Debug)]
pub struct Rule {
    pub selector: String, // TODO: parse w selectors
    pub declarations: Vec<Declaration>,
}

/// Parse a qualified rule
pub fn parse_rule<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Rule> {
    let selector_start = input.position();

    while !input.is_exhausted() {
        let token = input.next()?;
        if matches!(token, Token::CurlyBracketBlock) {
            break;
        }
    }

    let selector = input.slice_from(selector_start).trim();
    let selector = selector
        .strip_suffix("{")
        .unwrap_or(selector)
        .trim()
        .to_string();

    let declarations = input.parse_nested_block(|i| Ok(parse_declaration_block(i)))?;

    Ok(Rule {
        selector,
        declarations,
    })
}

fn parse_declaration_block<'i>(input: &mut Parser<'i, '_>) -> Vec<Declaration> {
    let mut declarations = Vec::new();

    loop {
        input.skip_whitespace();

        if input.is_exhausted() {
            break;
        }

        let result: ParseResult<'i, Vec<Declaration>> = input.try_parse(|i| {
            let decls = parse_declaration(i)?;

            let _ = i.try_parse(cssparser::Parser::expect_semicolon);
            Ok(decls)
        });

        match result {
            Ok(decls) => declarations.extend(decls),
            Err(_) => {
                while !input.is_exhausted() {
                    match input.next() {
                        Ok(Token::Semicolon) | Err(_) => break,
                        Ok(_) => {}
                    }
                }
            }
        }
    }

    declarations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::Property;
    use cssparser::ParserInput;

    fn parse(s: &str) -> Result<Rule, String> {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        parse_rule(&mut parser).map_err(|e| format!("{:?}", e.kind))
    }

    #[test]
    fn simple_rule() {
        let rule = parse(".container { display: flex }").expect("failed");
        assert_eq!(rule.selector, ".container");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, Property::Display);
    }

    #[test]
    fn multiple_declarations() {
        let rule = parse("div { color: red; margin: 10 }").expect("failed");
        assert_eq!(rule.selector, "div");
        // color: red = 1, margin: 10 = 4 (expanded)
        assert_eq!(rule.declarations.len(), 5);
    }

    #[test]
    fn complex_selector() {
        let rule = parse(".foo .bar > span:hover { color: cyan }").expect("failed");
        assert_eq!(rule.selector, ".foo .bar > span:hover");
    }

    #[test]
    fn multiple_selectors() {
        let rule = parse("div, .foo, #bar { display: none }").expect("failed");
        assert_eq!(rule.selector, "div, .foo, #bar");
    }

    #[test]
    fn trailing_semicolon() {
        let rule = parse(".test { color: red; }").expect("failed");
        assert_eq!(rule.declarations.len(), 1);
    }

    #[test]
    fn no_semicolon() {
        let rule = parse(".test { color: red }").expect("failed");
        assert_eq!(rule.declarations.len(), 1);
    }
}
