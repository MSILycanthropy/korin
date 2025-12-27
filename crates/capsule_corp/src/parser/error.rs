use cssparser::{ParseError, SourceLocation, ToCss, Token};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseErrorKind {
    #[error("expected integer, got float")]
    IntegerRequired,

    #[error("value {value} out of range ({min}..={max})")]
    OutOfRange { value: i64, min: i64, max: i64 },

    #[error("invalid length value")]
    InvalidLength,

    #[error("invalid color value")]
    InvalidColor,

    #[error("invalid variable")]
    InvalidVariable,

    #[error("invalid hex color format")]
    InvalidHexColor,

    #[error("unknown color '{0}'")]
    UnknownColor(String),

    #[error("unknown function '{0}'")]
    UnknownFunction(String),

    #[error("unknown property '{0}'")]
    UnknownProperty(String),

    #[error("unknown keyword '{keyword}' for property '{property}'")]
    UnknownKeyword {
        keyword: String,
        property: &'static str,
    },

    #[error("expected {what}, got {got}")]
    Expected { what: String, got: String },

    #[error("unexpected end of input")]
    UnexpectedEof,

    #[error("unexpected token '{0}'")]
    UnexpectedToken(String),

    #[error("failed to parse selector: {0}")]
    BadSelector(String),
}

pub type ParseResult<'i, T> = Result<T, ParseError<'i, ParseErrorKind>>;

pub const fn build_err<'i>(
    kind: ParseErrorKind,
    location: SourceLocation,
) -> ParseError<'i, ParseErrorKind> {
    ParseError {
        kind: cssparser::ParseErrorKind::Custom(kind),
        location,
    }
}

pub const fn error<'i, T>(kind: ParseErrorKind, location: SourceLocation) -> ParseResult<'i, T> {
    Err(build_err(kind, location))
}

pub fn expected<'i, T>(
    what: impl Into<String>,
    got: &Token<'_>,
    location: SourceLocation,
) -> ParseResult<'i, T> {
    error(
        ParseErrorKind::Expected {
            what: what.into(),
            got: got.to_css_string(),
        },
        location,
    )
}

pub fn unexpected_token<'i, T>(token: &Token<'_>, location: SourceLocation) -> ParseResult<'i, T> {
    error(
        ParseErrorKind::UnexpectedToken(token.to_css_string()),
        location,
    )
}

pub const fn integer_required<'i, T>(location: SourceLocation) -> ParseResult<'i, T> {
    error(ParseErrorKind::IntegerRequired, location)
}
