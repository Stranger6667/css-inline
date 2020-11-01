//! Python bindings for css-inline
#![warn(
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::print_stdout,
    clippy::integer_arithmetic,
    clippy::cast_possible_truncation,
    clippy::unwrap_used,
    clippy::map_unwrap_or,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
use css_inline as rust_inline;
use pyo3::{create_exception, exceptions, prelude::*, types::PyList, wrap_pyfunction};
use rayon::prelude::*;
use std::borrow::Cow;

const INLINE_ERROR_DOCSTRING: &str = "An error that can occur during CSS inlining";

create_exception!(css_inline, InlineError, exceptions::PyValueError);

struct InlineErrorWrapper(rust_inline::InlineError);

impl From<InlineErrorWrapper> for PyErr {
    fn from(error: InlineErrorWrapper) -> Self {
        match error.0 {
            rust_inline::InlineError::IO(error) => InlineError::new_err(error.to_string()),
            rust_inline::InlineError::Network(error) => InlineError::new_err(error.to_string()),
            rust_inline::InlineError::ParseError(message) => {
                InlineError::new_err(message.to_string())
            }
        }
    }
}

struct UrlError(url::ParseError);

impl From<UrlError> for PyErr {
    fn from(error: UrlError) -> Self {
        exceptions::PyValueError::new_err(error.0.to_string())
    }
}

fn parse_url(url: Option<String>) -> PyResult<Option<url::Url>> {
    Ok(if let Some(url) = url {
        Some(url::Url::parse(url.as_str()).map_err(UrlError)?)
    } else {
        None
    })
}

/// CSSInliner(inline_style_tags=True, remove_style_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None)
///
/// Customizable CSS inliner.
#[pyclass]
#[text_signature = "(inline_style_tags=True, remove_style_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None)"]
struct CSSInliner {
    inner: rust_inline::CSSInliner<'static>,
}

#[pymethods]
impl CSSInliner {
    #[new]
    fn new(
        inline_style_tags: Option<bool>,
        remove_style_tags: Option<bool>,
        base_url: Option<String>,
        load_remote_stylesheets: Option<bool>,
        extra_css: Option<String>,
    ) -> PyResult<Self> {
        let options = rust_inline::InlineOptions {
            inline_style_tags: inline_style_tags.unwrap_or(true),
            remove_style_tags: remove_style_tags.unwrap_or(false),
            base_url: parse_url(base_url)?,
            load_remote_stylesheets: load_remote_stylesheets.unwrap_or(true),
            extra_css: extra_css.map(Cow::Owned),
        };
        Ok(CSSInliner {
            inner: rust_inline::CSSInliner::new(options),
        })
    }

    /// inline(html)
    ///
    /// Inline CSS in the given HTML document
    #[text_signature = "(html)"]
    fn inline(&self, html: &str) -> PyResult<String> {
        Ok(self.inner.inline(html).map_err(InlineErrorWrapper)?)
    }

    /// inline_many(htmls)
    ///
    /// Inline CSS in multiple HTML documents
    #[text_signature = "(htmls)"]
    fn inline_many(&self, htmls: &PyList) -> PyResult<Vec<String>> {
        inline_many_impl(&self.inner, htmls)
    }
}

/// inline(html, inline_style_tags=True, remove_style_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None)
///
/// Inline CSS in the given HTML document
#[pyfunction]
#[text_signature = "(html, inline_style_tags=True, remove_style_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None)"]
fn inline(
    html: &str,
    inline_style_tags: Option<bool>,
    remove_style_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
    extra_css: Option<&str>,
) -> PyResult<String> {
    let options = rust_inline::InlineOptions {
        inline_style_tags: inline_style_tags.unwrap_or(true),
        remove_style_tags: remove_style_tags.unwrap_or(false),
        base_url: parse_url(base_url)?,
        load_remote_stylesheets: load_remote_stylesheets.unwrap_or(true),
        extra_css: extra_css.map(Cow::Borrowed),
    };
    let inliner = rust_inline::CSSInliner::new(options);
    Ok(inliner.inline(html).map_err(InlineErrorWrapper)?)
}

/// inline_many(htmls, inline_style_tags=True, remove_style_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None)
///
/// Inline CSS in multiple HTML documents
#[pyfunction]
#[text_signature = "(htmls, inline_style_tags=True, remove_style_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None)"]
fn inline_many(
    htmls: &PyList,
    inline_style_tags: Option<bool>,
    remove_style_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
    extra_css: Option<&str>,
) -> PyResult<Vec<String>> {
    let options = rust_inline::InlineOptions {
        inline_style_tags: inline_style_tags.unwrap_or(true),
        remove_style_tags: remove_style_tags.unwrap_or(false),
        base_url: parse_url(base_url)?,
        load_remote_stylesheets: load_remote_stylesheets.unwrap_or(true),
        extra_css: extra_css.map(Cow::Borrowed),
    };
    let inliner = rust_inline::CSSInliner::new(options);
    inline_many_impl(&inliner, htmls)
}

fn inline_many_impl(inliner: &rust_inline::CSSInliner, htmls: &PyList) -> PyResult<Vec<String>> {
    // Extract strings from the list. It will fail if there is any non-string value
    let extracted: Result<Vec<_>, _> = htmls.iter().map(|item| item.extract::<&str>()).collect();
    let inlined: Result<Vec<_>, _> = extracted?
        .par_iter()
        .map(|html| inliner.inline(html))
        .collect();
    Ok(inlined.map_err(InlineErrorWrapper)?)
}

#[allow(dead_code)]
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// Fast CSS inlining written in Rust
#[pymodule]
fn css_inline(py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<CSSInliner>()?;
    module.add_wrapped(wrap_pyfunction!(inline))?;
    module.add_wrapped(wrap_pyfunction!(inline_many))?;
    let inline_error = py.get_type::<InlineError>();
    inline_error.setattr("__doc__", INLINE_ERROR_DOCSTRING)?;
    module.add("InlineError", inline_error)?;
    module.add("__build__", pyo3_built::pyo3_built!(py, build))?;
    Ok(())
}
