use crate::{InlineError, Result};
use futures_util::FutureExt;
use std::io::ErrorKind;

type AsyncResult<'a, T> = std::pin::Pin<Box<dyn futures_util::Future<Output = Result<T>> + 'a>>;

/// Resolving stylesheets from various sources.
pub trait StylesheetResolver: Send + Sync {
    /// Retrieve a stylesheet from a network or local filesystem location in a blocking way.
    ///
    /// # Errors
    ///
    /// Any network or filesystem related error, or an error during response parsing.
    fn retrieve_blocking(&self, location: &str) -> Result<String> {
        if location.starts_with("https") | location.starts_with("http") {
            #[cfg(feature = "http-blocking")]
            {
                self.retrieve_from_url_blocking(location)
            }
            #[cfg(not(feature = "http-blocking"))]
            {
                Err(InlineError::IO(std::io::Error::new(
                    ErrorKind::Unsupported,
                    "Loading external URLs requires the `http-blocking` feature",
                )))
            }
        } else {
            #[cfg(feature = "file")]
            {
                self.retrieve_from_path(location)
            }
            #[cfg(not(feature = "file"))]
            {
                Err(InlineError::IO(std::io::Error::new(
                    ErrorKind::Unsupported,
                    "Loading local files requires the `file` feature",
                )))
            }
        }
    }
    /// Retrieve a stylesheet from a network location in a blocking way.
    ///
    /// # Errors
    ///
    /// Any network-related error, or an error during response parsing.
    fn retrieve_from_url_blocking(&self, url: &str) -> Result<String> {
        Err(self.unsupported(&format!("Loading external URLs is not supported: {url}")))
    }
    /// Retrieve a stylesheet from a network or local filesystem location in a non-blocking way.
    ///
    /// # Errors
    ///
    /// Any network or filesystem related error, or an error during response parsing.
    fn retrieve<'a>(&'a self, location: &'a str) -> AsyncResult<'_, String> {
        if location.starts_with("https") | location.starts_with("http") {
            #[cfg(feature = "http")]
            {
                self.retrieve_from_url(location)
            }
            #[cfg(not(feature = "http"))]
            {
                async move {
                    Err(InlineError::IO(std::io::Error::new(
                        ErrorKind::Unsupported,
                        "Loading external URLs requires the `http` feature",
                    )))
                }
                .boxed_local()
            }
        } else {
            async move {
                #[cfg(feature = "file")]
                {
                    self.retrieve_from_path(location)
                }
                #[cfg(not(feature = "file"))]
                {
                    Err(InlineError::IO(std::io::Error::new(
                        ErrorKind::Unsupported,
                        "Loading local files requires the `file` feature",
                    )))
                }
            }
            .boxed_local()
        }
    }
    /// Retrieve a stylesheet from a network location in a non-blocking way.
    ///
    /// # Errors
    ///
    /// Any network-related error, or an error during response parsing.
    fn retrieve_from_url<'a>(&'a self, url: &'a str) -> AsyncResult<'_, String> {
        async move { Err(self.unsupported(&format!("Loading external URLs is not supported: {url}"))) }.boxed_local()
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
        InlineError::IO(std::io::Error::new(ErrorKind::Unsupported, reason))
    }
}

/// Default stylesheet resolver.
#[derive(Debug, Default)]
pub struct DefaultStylesheetResolver;

impl StylesheetResolver for DefaultStylesheetResolver {
    #[cfg(feature = "http")]
    fn retrieve_from_url<'a>(&'a self, url: &'a str) -> AsyncResult<'_, String> {
        let into_error = |error| InlineError::Network {
            error,
            location: url.to_string(),
        };
        async move {
            reqwest::get(url)
                .await
                .map_err(into_error)?
                .text()
                .await
                .map_err(into_error)
        }
        .boxed_local()
    }
    #[cfg(feature = "http-blocking")]
    fn retrieve_from_url_blocking(&self, url: &str) -> Result<String> {
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
