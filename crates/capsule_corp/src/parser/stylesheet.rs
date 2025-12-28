use cssparser::{Parser, ParserInput, StyleSheetParser};

use crate::{
    ParseResult,
    parser::rule::{Rule, TopLevelRuleParser},
};

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
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
    let mut rule_parser = TopLevelRuleParser;

    let rules = StyleSheetParser::new(input, &mut rule_parser);

    for rule in rules {
        match rule {
            Ok(rule) => {
                stylesheet.rules.push(rule);
            }
            Err((_err, _slice)) => {
                // TODO: Logging
            }
        }
    }

    Ok(stylesheet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CustomValue, Property};
    use ginyu_force::Pose;

    fn get_custom_property<'a>(rule: &'a Rule, name: &str) -> Option<&'a str> {
        let pose = Pose::from(name);
        rule.declarations.iter().find_map(|d| {
            if d.property == Property::Custom(pose) {
                d.value.as_custom().and_then(|c| match c {
                    CustomValue::Resolved(s) => Some(s.as_str()),
                    _ => None,
                })
            } else {
                None
            }
        })
    }

    fn count_custom_properties(rule: &Rule) -> usize {
        rule.declarations
            .iter()
            .filter(|d| d.property.is_custom())
            .count()
    }

    #[test]
    fn empty_stylesheet() {
        let stylesheet = Stylesheet::parse("").expect("failed");
        assert!(stylesheet.rules.is_empty());
    }

    #[test]
    fn single_rule() {
        let stylesheet = Stylesheet::parse(".foo { display: flex }").expect("failed");
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors.len(), 1);
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
    fn root_with_custom_properties() {
        let stylesheet = Stylesheet::parse(
            r"
            :root {
                --primary: blue;
                --spacing: 2;
            }
            .foo { display: flex }
        ",
        )
        .expect("failed");
        assert_eq!(stylesheet.rules.len(), 2);
        assert_eq!(count_custom_properties(&stylesheet.rules[0]), 2);
        assert_eq!(
            get_custom_property(&stylesheet.rules[0], "primary"),
            Some("blue")
        );
        assert_eq!(
            get_custom_property(&stylesheet.rules[0], "spacing"),
            Some("2")
        );
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
    }

    #[test]
    fn selector_list() {
        let stylesheet = Stylesheet::parse(
            r"
            div, .foo, #bar { display: flex }
        ",
        )
        .expect("failed");
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selectors.len(), 3);
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

    #[test]
    fn nested_rules_in_stylesheet() {
        let stylesheet = Stylesheet::parse(
            r"
            .parent {
                color: red;
                .child { color: blue }
            }
        ",
        )
        .expect("failed");
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].nested_rules.len(), 1);
    }

    #[test]
    fn custom_properties_throughout() {
        let stylesheet = Stylesheet::parse(
            r"
            :root { --bg: #1a1a2e }
            .button { --accent: red; color: var(--accent) }
        ",
        )
        .expect("failed");
        assert_eq!(stylesheet.rules.len(), 2);
        assert_eq!(count_custom_properties(&stylesheet.rules[0]), 1);
        assert_eq!(
            get_custom_property(&stylesheet.rules[0], "bg"),
            Some("#1a1a2e")
        );
        assert_eq!(count_custom_properties(&stylesheet.rules[1]), 1);
        assert_eq!(
            get_custom_property(&stylesheet.rules[1], "accent"),
            Some("red")
        );
    }
}
