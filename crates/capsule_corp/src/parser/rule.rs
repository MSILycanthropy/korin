use cssparser::{
    AtRuleParser, CowRcStr, DeclarationParser, Parser, ParserState, QualifiedRuleParser,
    RuleBodyItemParser, RuleBodyParser,
};
use selectors::SelectorList;

use crate::{
    ParseErrorKind, ParseResult, Selectors,
    parser::{
        declaration::{Declaration, parse_declaration},
        selector::{parse_selector, parse_selector_for_nesting},
    },
};

#[derive(Clone, Debug)]
pub struct Rule {
    pub selectors: SelectorList<Selectors>,
    pub declarations: Vec<Declaration>,
    pub nested_rules: Vec<Rule>,
}

impl Rule {
    pub const fn new(selectors: SelectorList<Selectors>, declarations: Vec<Declaration>) -> Self {
        Self {
            selectors,
            declarations,
            nested_rules: Vec::new(),
        }
    }
}

enum RuleBodyItem {
    Declarations(Vec<Declaration>),
    NestedRule(Rule),
}

struct RuleParser;

impl<'i> DeclarationParser<'i> for RuleParser {
    type Declaration = RuleBodyItem;
    type Error = ParseErrorKind;

    fn parse_value<'t>(
        &mut self,
        name: CowRcStr<'i>,
        input: &mut Parser<'i, 't>,
        _declaration_start: &ParserState,
    ) -> ParseResult<'i, Self::Declaration> {
        let declarations = parse_declaration(&name, input)?;
        Ok(RuleBodyItem::Declarations(declarations))
    }
}

impl AtRuleParser<'_> for RuleParser {
    type Prelude = ();
    type AtRule = RuleBodyItem;
    type Error = ParseErrorKind;
}

impl<'i> QualifiedRuleParser<'i> for RuleParser {
    type Prelude = SelectorList<Selectors>;
    type QualifiedRule = RuleBodyItem;
    type Error = ParseErrorKind;

    fn parse_prelude<'t>(&mut self, input: &mut Parser<'i, 't>) -> ParseResult<'i, Self::Prelude> {
        parse_selector_for_nesting(input)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> ParseResult<'i, Self::QualifiedRule> {
        let rule = parse_rule_body(prelude, input);

        Ok(RuleBodyItem::NestedRule(rule))
    }
}

impl RuleBodyItemParser<'_, RuleBodyItem, ParseErrorKind> for RuleParser {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

pub struct TopLevelRuleParser;

impl AtRuleParser<'_> for TopLevelRuleParser {
    type Prelude = ();
    type AtRule = Rule;
    type Error = ParseErrorKind;
}

impl<'i> QualifiedRuleParser<'i> for TopLevelRuleParser {
    type Prelude = SelectorList<Selectors>;
    type QualifiedRule = Rule;
    type Error = ParseErrorKind;

    fn parse_prelude<'t>(&mut self, input: &mut Parser<'i, 't>) -> ParseResult<'i, Self::Prelude> {
        parse_selector(input)
    }

    fn parse_block<'t>(
        &mut self,
        prelude: Self::Prelude,
        _start: &ParserState,
        input: &mut Parser<'i, 't>,
    ) -> ParseResult<'i, Self::QualifiedRule> {
        Ok(parse_rule_body(prelude, input))
    }
}

fn parse_rule_body(selectors: SelectorList<Selectors>, input: &mut Parser<'_, '_>) -> Rule {
    let mut declarations = Vec::new();
    let mut nested_rules = Vec::new();

    let mut parser = RuleParser;
    let items = RuleBodyParser::new(input, &mut parser);

    for result in items {
        match result {
            Ok(RuleBodyItem::Declarations(decls)) => {
                declarations.extend(decls);
            }
            Ok(RuleBodyItem::NestedRule(rule)) => {
                nested_rules.push(rule);
            }
            Err((_err, _slice)) => {
                // eprintln!("skipping invalid rule body item: {:?}", err);
            }
        }
    }

    Rule {
        selectors,
        declarations,
        nested_rules,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::property::Property;
    use cssparser::{ParserInput, StyleSheetParser};
    use ginyu_force::Pose;

    fn parse(s: &str) -> Result<Rule, String> {
        let mut input = ParserInput::new(s);
        let mut parser = Parser::new(&mut input);
        let mut rule_parser = TopLevelRuleParser;

        let mut iter = StyleSheetParser::new(&mut parser, &mut rule_parser);
        match iter.next() {
            Some(Ok(rule)) => Ok(rule),
            Some(Err((e, _))) => Err(format!("{e:?}")),
            None => Err("no rule found".to_string()),
        }
    }

    fn get_custom_property<'a>(rule: &'a Rule, name: &str) -> Option<&'a str> {
        let pose = Pose::from(name);
        rule.declarations.iter().find_map(|d| {
            if d.property == Property::Custom(pose) {
                d.value.as_custom().and_then(|c| match c {
                    crate::CustomValue::Resolved(s) => Some(s.as_str()),
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
    fn simple_rule() {
        let rule = parse(".container { display: flex }").expect("parse failed");
        assert_eq!(rule.selectors.len(), 1);
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.declarations[0].property, Property::Display);
        assert!(rule.nested_rules.is_empty());
    }

    #[test]
    fn multiple_declarations() {
        let rule = parse("div { color: red; margin: 10 }").expect("parse failed");
        // color: red = 1, margin: 10 = 4 (expanded)
        assert_eq!(rule.declarations.len(), 5);
    }

    #[test]
    fn complex_selector() {
        let rule = parse(".foo .bar > span:hover { color: cyan }").expect("parse failed");
        assert_eq!(rule.selectors.len(), 1);
    }

    #[test]
    fn multiple_selectors() {
        let rule = parse("div, .foo, #bar { display: none }").expect("parse failed");
        assert_eq!(rule.selectors.len(), 3);
    }

    #[test]
    fn trailing_semicolon() {
        let rule = parse(".test { color: red; }").expect("parse failed");
        assert_eq!(rule.declarations.len(), 1);
    }

    #[test]
    fn no_semicolon() {
        let rule = parse(".test { color: red }").expect("parse failed");
        assert_eq!(rule.declarations.len(), 1);
    }

    #[test]
    fn custom_property_simple() {
        let rule = parse(".foo { --primary: red }").expect("parse failed");
        assert_eq!(count_custom_properties(&rule), 1);
        assert_eq!(get_custom_property(&rule, "primary"), Some("red"));
    }

    #[test]
    fn custom_property_complex_value() {
        let rule = parse(".foo { --spacing: calc(100% - 10) }").expect("parse failed");
        assert_eq!(count_custom_properties(&rule), 1);
        assert_eq!(
            get_custom_property(&rule, "spacing"),
            Some("calc(100% - 10)")
        );
    }

    #[test]
    fn custom_property_with_fallback() {
        let rule = parse(".foo { --color: var(--other, blue) }").expect("parse failed");
        assert_eq!(count_custom_properties(&rule), 1);
        // This one has a var() so it's unresolved, not resolved
        let pose = Pose::from("color");
        let decl = rule
            .declarations
            .iter()
            .find(|d| d.property == Property::Custom(pose))
            .expect("missing --color");
        assert!(decl.value.as_custom().is_some());
    }

    #[test]
    fn mixed_properties_and_custom() {
        let rule = parse(".foo { --primary: blue; color: red; margin: 1 }").expect("parse failed");
        assert_eq!(count_custom_properties(&rule), 1);
        // custom + color + margin (4 expanded) = 6
        assert_eq!(rule.declarations.len(), 6);
    }

    #[test]
    fn multiple_custom_properties() {
        let rule = parse(".foo { --a: 1; --b: 2; --c: 3 }").expect("parse failed");
        assert_eq!(count_custom_properties(&rule), 3);
        assert_eq!(get_custom_property(&rule, "a"), Some("1"));
        assert_eq!(get_custom_property(&rule, "b"), Some("2"));
        assert_eq!(get_custom_property(&rule, "c"), Some("3"));
    }

    #[test]
    fn custom_property_last_wins() {
        // Both declarations are present, cascade determines which wins at compute time
        let rule = parse(".foo { --x: first; --x: second }").expect("parse failed");
        assert_eq!(count_custom_properties(&rule), 2);
    }

    #[test]
    fn custom_property_in_nested_rule() {
        let rule = parse(".foo { .bar { --nested: value } }").expect("parse failed");
        assert_eq!(count_custom_properties(&rule), 0);
        assert_eq!(rule.nested_rules.len(), 1);
        assert_eq!(count_custom_properties(&rule.nested_rules[0]), 1);
        assert_eq!(
            get_custom_property(&rule.nested_rules[0], "nested"),
            Some("value")
        );
    }

    #[test]
    fn nested_rule_with_class() {
        let rule =
            parse(".container { color: red; .child { color: blue } }").expect("parse failed");
        assert_eq!(rule.declarations.len(), 1);
        assert_eq!(rule.nested_rules.len(), 1);
        assert_eq!(rule.nested_rules[0].declarations.len(), 1);
    }

    #[test]
    fn nested_rule_with_ampersand() {
        let rule = parse(".btn { &:hover { color: blue } }").expect("parse failed");
        assert_eq!(rule.declarations.len(), 0);
        assert_eq!(rule.nested_rules.len(), 1);
    }

    #[test]
    fn nested_rule_with_combinator() {
        let rule = parse(".foo { > .bar { color: red } }").expect("parse failed");
        assert_eq!(rule.nested_rules.len(), 1);
    }

    #[test]
    fn deeply_nested() {
        let rule = parse(".a { .b { .c { color: red } } }").expect("parse failed");
        assert_eq!(rule.nested_rules.len(), 1);
        assert_eq!(rule.nested_rules[0].nested_rules.len(), 1);
        assert_eq!(rule.nested_rules[0].nested_rules[0].declarations.len(), 1);
    }

    #[test]
    fn mixed_declarations_and_nesting() {
        let rule =
            parse(".foo { color: red; &:hover { color: blue }; margin: 1 }").expect("parse failed");
        assert_eq!(rule.declarations.len(), 5); // color + margin (4 expanded)
        assert_eq!(rule.nested_rules.len(), 1);
    }

    #[test]
    fn multiple_nested_rules() {
        let rule = parse(".parent { .child1 { color: red } .child2 { color: blue } }")
            .expect("parse failed");
        assert_eq!(rule.nested_rules.len(), 2);
    }

    #[test]
    fn nested_with_multiple_selectors() {
        let rule = parse(".foo { .bar, .baz { color: red } }").expect("parse failed");
        assert_eq!(rule.nested_rules.len(), 1);
        assert_eq!(rule.nested_rules[0].selectors.len(), 2);
    }

    #[test]
    fn ampersand_in_middle() {
        let rule = parse(".foo { .parent & { color: red } }").expect("parse failed");
        assert_eq!(rule.nested_rules.len(), 1);
    }

    #[test]
    fn multiple_ampersands() {
        let rule = parse(".foo { & + & { color: red } }").expect("parse failed");
        assert_eq!(rule.nested_rules.len(), 1);
    }
}
