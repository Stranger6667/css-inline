//! Python bindings for css-inline
#![warn(
    clippy::pedantic,
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
    variant_size_differences,
    rust_2018_idioms,
    rust_2018_compatibility
)]
use ::css_inline as rust_inline;
use pyo3::{create_exception, exceptions, prelude::*, types::PyList, wrap_pyfunction};
use rayon::prelude::*;
use std::borrow::Cow;
#[macro_use]
extern crate pyo3_built;

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
            rust_inline::InlineError::MissingStyleSheet { .. } => {
                InlineError::new_err(error.0.to_string())
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

/// CSSInliner(keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None, preallocate_node_capacity=32)
///
/// Customizable CSS inliner.
#[pyclass]
struct CSSInliner {
    inner: rust_inline::CSSInliner<'static>,
}

#[pymethods]
impl CSSInliner {
    #[new(
        text_signature = "(keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None, preallocate_node_capacity=32)"
    )]
    fn new(
        keep_style_tags: Option<bool>,
        keep_link_tags: Option<bool>,
        base_url: Option<String>,
        load_remote_stylesheets: Option<bool>,
        extra_css: Option<String>,
        preallocate_node_capacity: Option<usize>,
    ) -> PyResult<Self> {
        let options = rust_inline::InlineOptions {
            keep_style_tags: keep_style_tags.unwrap_or(false),
            keep_link_tags: keep_link_tags.unwrap_or(false),
            base_url: parse_url(base_url)?,
            load_remote_stylesheets: load_remote_stylesheets.unwrap_or(true),
            extra_css: extra_css.map(Cow::Owned),
            preallocate_node_capacity: preallocate_node_capacity.unwrap_or(32),
        };
        Ok(CSSInliner {
            inner: rust_inline::CSSInliner::new(options),
        })
    }

    /// inline(html)
    ///
    /// Inline CSS in the given HTML document
    #[pyo3(text_signature = "(html)")]
    fn inline(&self, html: &str) -> PyResult<String> {
        Ok(self.inner.inline(html).map_err(InlineErrorWrapper)?)
    }

    /// inline_many(htmls)
    ///
    /// Inline CSS in multiple HTML documents
    #[pyo3(text_signature = "(htmls)")]
    fn inline_many(&self, htmls: &PyList) -> PyResult<Vec<String>> {
        inline_many_impl(&self.inner, htmls)
    }
}

/// inline(html, keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None, preallocate_node_capacity=32)
///
/// Inline CSS in the given HTML document
#[pyfunction]
#[pyo3(
    text_signature = "(html, keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None, preallocate_node_capacity=32)"
)]
fn inline(
    html: &str,
    keep_style_tags: Option<bool>,
    keep_link_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
    extra_css: Option<&str>,
    preallocate_node_capacity: Option<usize>,
) -> PyResult<String> {
    let options = rust_inline::InlineOptions {
        keep_style_tags: keep_style_tags.unwrap_or(false),
        keep_link_tags: keep_link_tags.unwrap_or(false),
        base_url: parse_url(base_url)?,
        load_remote_stylesheets: load_remote_stylesheets.unwrap_or(true),
        extra_css: extra_css.map(Cow::Borrowed),
        preallocate_node_capacity: preallocate_node_capacity.unwrap_or(32),
    };
    let inliner = rust_inline::CSSInliner::new(options);
    Ok(inliner.inline(html).map_err(InlineErrorWrapper)?)
}

/// inline_many(htmls, keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None, preallocate_node_capacity=32)
///
/// Inline CSS in multiple HTML documents
#[pyfunction]
#[pyo3(
    text_signature = "(htmls, keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, extra_css=None, preallocate_node_capacity=32)"
)]
fn inline_many(
    htmls: &PyList,
    keep_style_tags: Option<bool>,
    keep_link_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
    extra_css: Option<&str>,
    preallocate_node_capacity: Option<usize>,
) -> PyResult<Vec<String>> {
    let options = rust_inline::InlineOptions {
        keep_style_tags: keep_style_tags.unwrap_or(false),
        keep_link_tags: keep_link_tags.unwrap_or(false),
        base_url: parse_url(base_url)?,
        load_remote_stylesheets: load_remote_stylesheets.unwrap_or(true),
        extra_css: extra_css.map(Cow::Borrowed),
        preallocate_node_capacity: preallocate_node_capacity.unwrap_or(32),
    };
    let inliner = rust_inline::CSSInliner::new(options);
    inline_many_impl(&inliner, htmls)
}

fn inline_many_impl(
    inliner: &rust_inline::CSSInliner<'_>,
    htmls: &PyList,
) -> PyResult<Vec<String>> {
    // Extract strings from the list. It will fail if there is any non-string value
    let extracted: Result<Vec<_>, _> = htmls.iter().map(pyo3::PyAny::extract::<&str>).collect();
    let output: Result<Vec<_>, _> = extracted?
        .par_iter()
        .map(|html| inliner.inline(html))
        .collect();
    Ok(output.map_err(InlineErrorWrapper)?)
}

#[allow(dead_code)]
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// Fast CSS inlining written in Rust
#[pymodule]
fn css_inline(py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<CSSInliner>()?;
    module.add_wrapped(wrap_pyfunction!(inline))?;
    module.add_wrapped(wrap_pyfunction!(inline_many))?;
    let inline_error = py.get_type::<InlineError>();
    inline_error.setattr("__doc__", INLINE_ERROR_DOCSTRING)?;
    module.add("InlineError", inline_error)?;
    module.add("__build__", pyo3_built::pyo3_built!(py, build))?;
    Ok(())
}
