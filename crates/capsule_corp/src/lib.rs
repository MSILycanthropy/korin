mod brief;
mod bulma;
mod document;
mod macros;
mod parser;
mod property;
mod values;

pub use brief::*;
pub use bulma::*;
use cssparser::ParserInput;
pub use document::*;
pub use ginyu_force::Pose;

pub use parser::{ParseErrorKind, ParseResult, Stylesheet, parse_stylesheet};
pub use property::*;
pub use values::*;

pub type SelectorList = selectors::SelectorList<Selectors>;

pub fn parse_selector(selector: &str) -> Result<SelectorList, String> {
    let mut input = ParserInput::new(selector);
    let mut parser = cssparser::Parser::new(&mut input);
    parser::parse_selector(&mut parser).map_err(|err| format!("{:?}", err.kind))
}
