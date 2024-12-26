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
    /// Missing stylesheet file.
    MissingStyleSheet {
        /// Path to the missing file.
        path: String,
    },
    /// Input-output error. May happen during writing the resulting HTML or retrieving a stylesheet
    /// from the filesystem.
    IO(io::Error),
    /// Network-related problem. E.g. resource is not available.
    #[cfg(feature = "http")]
    Network {
        /// Original network error.
        error: reqwest::Error,
        /// The stylesheet location caused the error.
        location: String,
    },
    /// Syntax errors or unsupported selectors.
    ParseError(Cow<'static, str>),
}

impl From<io::Error> for InlineError {
    fn from(error: io::Error) -> Self {
        Self::IO(error)
    }
}

impl Error for InlineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InlineError::IO(error) => Some(error),
            #[cfg(feature = "http")]
            InlineError::Network { error, .. } => Some(error),
            InlineError::MissingStyleSheet { .. } | InlineError::ParseError(_) => None,
        }
    }
}

impl Display for InlineError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(error) => error.fmt(f),
            #[cfg(feature = "http")]
            Self::Network { error, location } => f.write_fmt(format_args!("{error}: {location}")),
            Self::ParseError(error) => f.write_str(error),
            Self::MissingStyleSheet { path } => {
                f.write_fmt(format_args!("Missing stylesheet file: {path}"))
            }
        }
    }
}

impl From<(ParseError<'_, ()>, &str)> for InlineError {
    fn from(error: (ParseError<'_, ()>, &str)) -> Self {
        match error.0.kind {
            ParseErrorKind::Basic(kind) => match kind {
                BasicParseErrorKind::UnexpectedToken(token) => {
                    Self::ParseError(Cow::Owned(format!("Unexpected token: {token:?}")))
                }
                BasicParseErrorKind::EndOfInput => Self::ParseError(Cow::Borrowed("End of input")),
                BasicParseErrorKind::AtRuleInvalid(value) => {
                    Self::ParseError(Cow::Owned(format!("Invalid @ rule: {value}")))
                }
                BasicParseErrorKind::AtRuleBodyInvalid => {
                    Self::ParseError(Cow::Borrowed("Invalid @ rule body"))
                }
                BasicParseErrorKind::QualifiedRuleInvalid => {
                    Self::ParseError(Cow::Borrowed("Invalid qualified rule"))
                }
            },
            ParseErrorKind::Custom(()) => Self::ParseError(Cow::Borrowed("Unknown error")),
        }
    }
}
