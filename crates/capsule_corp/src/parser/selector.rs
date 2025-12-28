use selectors::{SelectorList, parser::ParseRelative};

use crate::{ParseErrorKind, ParseResult, SelectorParser, Selectors, parser::error::build_err};

pub fn parse_selector<'i>(
    input: &mut cssparser::Parser<'i, '_>,
) -> ParseResult<'i, SelectorList<Selectors>> {
    parse_selector_inner(input, ParseRelative::No)
}

pub fn parse_selector_for_nesting<'i>(
    input: &mut cssparser::Parser<'i, '_>,
) -> ParseResult<'i, SelectorList<Selectors>> {
    parse_selector_inner(input, ParseRelative::ForNesting)
}

fn parse_selector_inner<'i>(
    input: &mut cssparser::Parser<'i, '_>,
    relative: ParseRelative,
) -> ParseResult<'i, SelectorList<Selectors>> {
    let location = input.current_source_location();
    SelectorList::parse(&SelectorParser, input, relative)
        .map_err(|e| build_err(ParseErrorKind::BadSelector(format!("{e:?}")), location))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cssparser::ParserInput;

    fn parse(s: &str) -> Result<SelectorList<Selectors>, String> {
        let mut input = ParserInput::new(s);
        let mut parser = cssparser::Parser::new(&mut input);
        parse_selector(&mut parser).map_err(|e| format!("{:?}", e.kind))
    }

    fn parse_nested(s: &str) -> Result<SelectorList<Selectors>, String> {
        let mut input = ParserInput::new(s);
        let mut parser = cssparser::Parser::new(&mut input);
        parse_selector_for_nesting(&mut parser).map_err(|e| format!("{:?}", e.kind))
    }

    #[test]
    fn parse_simple_selectors() {
        assert!(parse("div").is_ok());
        assert!(parse(".foo").is_ok());
        assert!(parse("#bar").is_ok());
        assert!(parse("[type]").is_ok());
    }

    #[test]
    fn parse_compound_selectors() {
        assert!(parse("div.foo").is_ok());
        assert!(parse("div.foo.bar").is_ok());
        assert!(parse("input[type=\"text\"]").is_ok());
    }

    #[test]
    fn parse_complex_selectors() {
        assert!(parse("div .foo").is_ok());
        assert!(parse("div > .foo").is_ok());
        assert!(parse("div + .foo").is_ok());
        assert!(parse("div ~ .foo").is_ok());
    }

    #[test]
    fn parse_pseudo_classes() {
        assert!(parse(":hover").is_ok());
        assert!(parse(":focus").is_ok());
        assert!(parse(".foo:hover").is_ok());
        assert!(parse(":first-child").is_ok());
        assert!(parse(":nth-child(2)").is_ok());
    }

    #[test]
    fn parse_selector_list() {
        assert!(parse("div, .foo, #bar").is_ok());
    }

    #[test]
    fn parse_ampersand_alone() {
        assert!(parse_nested("&").is_ok());
    }

    #[test]
    fn parse_ampersand_with_pseudo() {
        assert!(parse_nested("&:hover").is_ok());
        assert!(parse_nested("&:focus").is_ok());
    }

    #[test]
    fn parse_ampersand_with_class() {
        assert!(parse_nested("&.active").is_ok());
    }

    #[test]
    fn parse_ampersand_descendant() {
        assert!(parse_nested("& .child").is_ok());
    }

    #[test]
    fn parse_ampersand_child() {
        assert!(parse_nested("& > .child").is_ok());
    }

    #[test]
    fn parse_ampersand_in_middle() {
        assert!(parse_nested(".parent &").is_ok());
    }

    #[test]
    fn parse_multiple_ampersands() {
        assert!(parse_nested("& + &").is_ok());
    }

    #[test]
    fn parse_relative_selector() {
        // Relative selectors (start with combinator) are allowed in nesting
        assert!(parse_nested("> .child").is_ok());
        assert!(parse_nested("+ .sibling").is_ok());
    }
}
