//! Errors that may happen during inlining.
use cssparser::{BasicParseErrorKind, ParseError, ParseErrorKind};
use std::{
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
    ParseError(String),
}

impl From<io::Error> for InlineError {
    fn from(error: io::Error) -> Self {
        InlineError::IO(error)
    }
}
impl From<attohttpc::Error> for InlineError {
    fn from(error: attohttpc::Error) -> Self {
        InlineError::Network(error)
    }
}

impl Error for InlineError {}

impl Display for InlineError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            InlineError::IO(error) => write!(f, "{}", error),
            InlineError::Network(error) => write!(f, "{}", error),
            InlineError::ParseError(error) => write!(f, "{}", error),
        }
    }
}

impl From<(ParseError<'_, ()>, &str)> for InlineError {
    fn from(error: (ParseError<'_, ()>, &str)) -> Self {
        let message = match error.0.kind {
            ParseErrorKind::Basic(kind) => match kind {
                BasicParseErrorKind::UnexpectedToken(token) => {
                    format!("Unexpected token: {:?}", token)
                }
                BasicParseErrorKind::EndOfInput => "End of input".to_string(),
                BasicParseErrorKind::AtRuleInvalid(value) => format!("Invalid @ rule: {}", value),
                BasicParseErrorKind::AtRuleBodyInvalid => "Invalid @ rule body".to_string(),
                BasicParseErrorKind::QualifiedRuleInvalid => "Invalid qualified rule".to_string(),
            },
            ParseErrorKind::Custom(_) => "Unknown error".to_string(),
        };
        InlineError::ParseError(message)
    }
}
