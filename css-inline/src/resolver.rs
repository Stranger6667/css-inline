use crate::{InlineError, Result};
use std::io::ErrorKind;

/// Blocking way of resolving stylesheets from various sources.
pub trait StylesheetResolver: Send + Sync {
    /// Retrieve a stylesheet from a network or local filesystem location.
    ///
    /// # Errors
    ///
    /// Any network or filesystem related error, or an error during response parsing.
    fn retrieve(&self, location: &str) -> Result<String> {
        if location.starts_with("https") | location.starts_with("http") {
            #[cfg(feature = "http")]
            {
                self.retrieve_from_url(location)
            }

            #[cfg(not(feature = "http"))]
            {
                Err(std::io::Error::new(
                    ErrorKind::Unsupported,
                    "Loading external URLs requires the `http` feature",
                )
                .into())
            }
        } else {
            #[cfg(feature = "file")]
            {
                self.retrieve_from_path(location)
            }
            #[cfg(not(feature = "file"))]
            {
                Err(std::io::Error::new(
                    ErrorKind::Unsupported,
                    "Loading local files requires the `file` feature",
                )
                .into())
            }
        }
    }
    /// Retrieve a stylesheet from a network location.
    ///
    /// # Errors
    ///
    /// Any network-related error, or an error during response parsing.
    fn retrieve_from_url(&self, url: &str) -> Result<String> {
        Err(self.unsupported(&format!("Loading external URLs is not supported: {url}")))
    }
    /// Retrieve a stylesheet from the local filesystem.
    ///
    /// # Errors
    ///
    /// Any filesystem-related error.
    fn retrieve_from_path(&self, path: &str) -> Result<String> {
        let path = path.trim_start_matches("file://");
        std::fs::read_to_string(path).map_err(|error| match error.kind() {
            ErrorKind::NotFound => InlineError::MissingStyleSheet {
                path: path.to_string(),
            },
            #[cfg(target_family = "wasm")]
            ErrorKind::Unsupported => self.unsupported(&format!(
                "Loading local files is not supported on WASM: {path}"
            )),
            _ => InlineError::IO(error),
        })
    }
    /// Return the "Unsupported" kind of error.
    fn unsupported(&self, reason: &str) -> InlineError {
        std::io::Error::new(ErrorKind::Unsupported, reason).into()
    }
}

/// Default stylesheet resolver.
#[derive(Debug, Default)]
pub struct DefaultStylesheetResolver;

impl StylesheetResolver for DefaultStylesheetResolver {
    #[cfg(feature = "http")]
    fn retrieve_from_url(&self, url: &str) -> Result<String> {
        let into_error = |error| InlineError::Network {
            error,
            location: url.to_string(),
        };
        reqwest::blocking::get(url)
            .map_err(into_error)?
            .text()
            .map_err(into_error)
    }
}
