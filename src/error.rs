//! Errors that may happen during inlining.
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
