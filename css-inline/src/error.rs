//! Errors that may happen during inlining.
use cssparser::{BasicParseErrorKind, ParseError, ParseErrorKind};
use std::{
    borrow::Cow,
    error::Error,
    fmt,
    fmt::{Display, Formatter},
    io,
};

/// Inlining error
#[derive(Debug)]
pub enum InlineError {
    /// Input-output error. May happen during writing the resulting HTML.
    IO(io::Error),
    /// Network-related problem. E.g. resource is not available.
    Network(attohttpc::Error),
    /// Syntax errors or unsupported selectors.
    ParseError(Cow<'static, str>),
}

impl From<io::Error> for InlineError {
    fn from(error: io::Error) -> Self {
        Self::IO(error)
    }
}
impl From<attohttpc::Error> for InlineError {
    fn from(error: attohttpc::Error) -> Self {
        Self::Network(error)
    }
}

impl Error for InlineError {}

impl Display for InlineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(error) => error.fmt(f),
            Self::Network(error) => error.fmt(f),
            Self::ParseError(error) => f.write_str(error),
        }
    }
}

impl From<(ParseError<'_, ()>, &str)> for InlineError {
    fn from(error: (ParseError<'_, ()>, &str)) -> Self {
        return match error.0.kind {
            ParseErrorKind::Basic(kind) => match kind {
                BasicParseErrorKind::UnexpectedToken(token) => {
                    Self::ParseError(Cow::Owned(format!("Unexpected token: {:?}", token)))
                }
                BasicParseErrorKind::EndOfInput => Self::ParseError(Cow::Borrowed("End of input")),
                BasicParseErrorKind::AtRuleInvalid(value) => {
                    Self::ParseError(Cow::Owned(format!("Invalid @ rule: {}", value)))
                }
                BasicParseErrorKind::AtRuleBodyInvalid => {
                    Self::ParseError(Cow::Borrowed("Invalid @ rule body"))
                }
                BasicParseErrorKind::QualifiedRuleInvalid => {
                    Self::ParseError(Cow::Borrowed("Invalid qualified rule"))
                }
            },
            ParseErrorKind::Custom(_) => Self::ParseError(Cow::Borrowed("Unknown error")),
        };
    }
}
