use cssparser::{Parser, ParserInput, Token};
use rustc_hash::FxHashMap;

use crate::{
    ParseResult,
    parser::{
        declaration::Declaration,
        rule::{Rule, parse_rule},
    },
};

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
    pub variables: FxHashMap<String, String>,
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(source: &str) -> ParseResult<'_, Self> {
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);

        parse_stylesheet(&mut parser)
    }
}

pub fn parse_stylesheet<'i>(input: &mut Parser<'i, '_>) -> ParseResult<'i, Stylesheet> {
    let mut stylesheet = Stylesheet::new();

    loop {
        input.skip_whitespace();

        if input.is_exhausted() {
            break;
        }

        match input.try_parse(parse_rule) {
            Ok(rule) => {
                if rule.selector.trim() == ":root" {
                    extract_variables(&rule.declarations, &mut stylesheet.variables);
                } else {
                    stylesheet.rules.push(rule);
                }
            }
            Err(_) => {
                skip_to_next_rule(input);
            }
        }
    }

    Ok(stylesheet)
}

fn extract_variables(declarations: &[Declaration], _variables: &mut FxHashMap<String, String>) {
    for decl in declarations {
        // Variables are stored as custom properties (--name)
        // For now, we store the property name and value as strings
        // TODO: Handle custom properties properly
        let name = decl.property.to_name();
        if name.starts_with("--") {
            // We need to handle custom properties differently
            // For now, skip - we'll implement this properly later
        }
    }
}

fn skip_to_next_rule(input: &mut Parser) {
    let mut brace_depth = 0;

    loop {
        match input.next() {
            Ok(Token::CurlyBracketBlock) => {
                brace_depth += 1;

                let _ = input.parse_nested_block(|inner| {
                    while inner.next().is_ok() {}
                    Ok::<(), cssparser::ParseError<'_, ()>>(())
                });
                brace_depth -= 1;

                if brace_depth <= 0 {
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_stylesheet() {
        let stylesheet = Stylesheet::parse("").expect("failed");
        assert!(stylesheet.rules.is_empty());
        assert!(stylesheet.variables.is_empty());
    }

    #[test]
    fn single_rule() {
        let stylesheet = Stylesheet::parse(".foo { display: flex }").expect("failed");
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selector, ".foo");
    }

    #[test]
    fn multiple_rules() {
        let stylesheet = Stylesheet::parse(
            r"
            .foo { display: flex }
            .bar { color: red }
            #baz { margin: 10 }
        ",
        )
        .expect("failed");
        assert_eq!(stylesheet.rules.len(), 3);
        assert_eq!(stylesheet.rules[0].selector, ".foo");
        assert_eq!(stylesheet.rules[1].selector, ".bar");
        assert_eq!(stylesheet.rules[2].selector, "#baz");
    }

    #[test]
    fn rule_with_multiple_declarations() {
        let stylesheet = Stylesheet::parse(
            r"
            .container {
                display: flex;
                flex-direction: column;
                padding: 1 2;
                color: cyan;
            }
        ",
        )
        .expect("failed");
        assert_eq!(stylesheet.rules.len(), 1);
        // display + flex-direction + padding (4) + color = 7
        assert_eq!(stylesheet.rules[0].declarations.len(), 7);
    }

    #[test]
    fn root_rule_separate() {
        let stylesheet = Stylesheet::parse(
            r"
            :root {
                color: red;
            }
            .foo { display: flex }
        ",
        )
        .expect("failed");
        // :root should not be in rules
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selector, ".foo");
    }

    #[test]
    fn complex_selectors() {
        let stylesheet = Stylesheet::parse(
            r#"
            div.foo > span:hover { color: red }
            .a .b .c { display: none }
            [type="text"]:focus { border: solid cyan }
        "#,
        )
        .expect("failed");
        assert_eq!(stylesheet.rules.len(), 3);
        assert_eq!(stylesheet.rules[0].selector, "div.foo > span:hover");
        assert_eq!(stylesheet.rules[1].selector, ".a .b .c");
        assert_eq!(stylesheet.rules[2].selector, "[type=\"text\"]:focus");
    }

    #[test]
    fn recovers_from_invalid_rule() {
        let stylesheet = Stylesheet::parse(
            r"
            .valid { display: flex }
            .invalid { gobbledygook:: }
            .also-valid { color: red }
        ",
        )
        .expect("failed");
        // Should have at least the valid rules
        assert!(stylesheet.rules.len() >= 2);
    }
}
