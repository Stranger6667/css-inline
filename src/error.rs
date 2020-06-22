//! Errors that may happen during inlining.
use cssparser::ParseError;
use std::io;

/// Inlining error
#[derive(Debug)]
pub enum InlineError {
    /// Input-output error. May happen during writing the resulting HTML.
    IO(io::Error),
    /// Syntax errors or unsupported selectors.
    ParseError,
}

impl From<io::Error> for InlineError {
    fn from(error: io::Error) -> Self {
        InlineError::IO(error)
    }
}

impl From<(ParseError<'_, ()>, &str)> for InlineError {
    fn from(_: (ParseError<'_, ()>, &str)) -> Self {
        InlineError::ParseError
    }
}
