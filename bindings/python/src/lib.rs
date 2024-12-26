//! Python bindings for css-inline
#![warn(
    clippy::pedantic,
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
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
    variant_size_differences,
    rust_2018_idioms,
    rust_2018_compatibility,
    rust_2021_compatibility
)]
use std::num::NonZeroUsize;

use ::css_inline as rust_inline;
use pyo3::{create_exception, exceptions, prelude::*, types::PyList, wrap_pyfunction};
use rayon::prelude::*;
#[macro_use]
extern crate pyo3_built;

const INLINE_ERROR_DOCSTRING: &str = "An error that can occur during CSS inlining";

create_exception!(css_inline, InlineError, exceptions::PyValueError);

struct InlineErrorWrapper(rust_inline::InlineError);

impl From<InlineErrorWrapper> for PyErr {
    fn from(error: InlineErrorWrapper) -> Self {
        match error.0 {
            rust_inline::InlineError::IO(error) => InlineError::new_err(error.to_string()),
            #[cfg(not(target_arch = "wasm32"))]
            rust_inline::InlineError::Network { .. } => InlineError::new_err(error.0.to_string()),
            rust_inline::InlineError::ParseError(message) => {
                InlineError::new_err(message.to_string())
            }
            rust_inline::InlineError::MissingStyleSheet { .. } => {
                InlineError::new_err(error.0.to_string())
            }
        }
    }
}

struct UrlError {
    error: rust_inline::ParseError,
    url: String,
}

impl From<UrlError> for PyErr {
    fn from(error: UrlError) -> Self {
        exceptions::PyValueError::new_err(format!("{}: {}", error.error, error.url))
    }
}

fn parse_url(url: Option<String>) -> PyResult<Option<rust_inline::Url>> {
    Ok(if let Some(url) = url {
        Some(rust_inline::Url::parse(url.as_str()).map_err(|error| UrlError { error, url })?)
    } else {
        None
    })
}

/// StylesheetCache(size=8)
///
/// An LRU Cache for external stylesheets.
#[pyclass]
#[derive(Clone)]
struct StylesheetCache {
    size: NonZeroUsize,
}

#[pymethods]
impl StylesheetCache {
    #[new]
    #[pyo3(signature = (size))]
    fn new(size: Option<&Bound<'_, PyAny>>) -> PyResult<Self> {
        let size = if let Some(size) = size {
            const ERROR_MESSAGE: &str = "Cache size must be an integer greater than zero";
            let size = size
                .extract::<usize>()
                .map_err(|_| InlineError::new_err(ERROR_MESSAGE))?;
            NonZeroUsize::new(size).ok_or_else(|| InlineError::new_err(ERROR_MESSAGE))?
        } else {
            NonZeroUsize::new(8).expect("8 is not zero")
        };
        Ok(StylesheetCache { size })
    }
}

/// CSSInliner(inline_style_tags=True, keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, cache=None, extra_css=None, preallocate_node_capacity=32)
///
/// Customizable CSS inliner.
#[pyclass]
struct CSSInliner {
    inner: rust_inline::CSSInliner<'static>,
}

macro_rules! inliner {
    ($inline_style_tags:expr, $keep_style_tags:expr, $keep_link_tags:expr, $base_url:expr, $load_remote_stylesheets:expr, $cache:expr, $extra_css:expr, $preallocate_node_capacity:expr) => {{
        let options = rust_inline::InlineOptions {
            inline_style_tags: $inline_style_tags.unwrap_or(true),
            keep_style_tags: $keep_style_tags.unwrap_or(false),
            keep_link_tags: $keep_link_tags.unwrap_or(false),
            base_url: $crate::parse_url($base_url)?,
            load_remote_stylesheets: $load_remote_stylesheets.unwrap_or(true),
            cache: {
                if let Some(cache) = $cache {
                    Some(std::sync::Mutex::new(rust_inline::StylesheetCache::new(
                        cache.size,
                    )))
                } else {
                    None
                }
            },
            extra_css: $extra_css.map(Into::into),
            preallocate_node_capacity: $preallocate_node_capacity.unwrap_or(32),
            resolver: std::sync::Arc::new(rust_inline::DefaultStylesheetResolver),
        };
        rust_inline::CSSInliner::new(options)
    }};
}

#[pymethods]
impl CSSInliner {
    #[new]
    #[pyo3(
        signature = (inline_style_tags=true, keep_style_tags=false, keep_link_tags=false, base_url=None, load_remote_stylesheets=true, cache=None, extra_css=None, preallocate_node_capacity=32)
    )]
    #[allow(clippy::too_many_arguments)]
    fn new(
        inline_style_tags: Option<bool>,
        keep_style_tags: Option<bool>,
        keep_link_tags: Option<bool>,
        base_url: Option<String>,
        load_remote_stylesheets: Option<bool>,
        cache: Option<StylesheetCache>,
        extra_css: Option<String>,
        preallocate_node_capacity: Option<usize>,
    ) -> PyResult<Self> {
        let inner = inliner!(
            inline_style_tags,
            keep_style_tags,
            keep_link_tags,
            base_url,
            load_remote_stylesheets,
            cache,
            extra_css,
            preallocate_node_capacity
        );
        Ok(CSSInliner { inner })
    }

    /// inline(html)
    ///
    /// Inline CSS in the given HTML document
    #[pyo3(text_signature = "(html)")]
    fn inline(&self, html: &str) -> PyResult<String> {
        Ok(self.inner.inline(html).map_err(InlineErrorWrapper)?)
    }

    /// inline_fragment(html, css)
    ///
    /// Inline CSS into the given HTML fragment
    #[pyo3(text_signature = "(html, css)")]
    fn inline_fragment(&self, html: &str, css: &str) -> PyResult<String> {
        Ok(self
            .inner
            .inline_fragment(html, css)
            .map_err(InlineErrorWrapper)?)
    }

    /// inline_many(htmls)
    ///
    /// Inline CSS in multiple HTML documents
    #[pyo3(text_signature = "(htmls)")]
    fn inline_many(&self, htmls: &Bound<'_, PyList>) -> PyResult<Vec<String>> {
        inline_many_impl(&self.inner, htmls)
    }

    /// inline_many_fragments(htmls, css)
    ///
    /// Inline CSS in multiple HTML documents
    #[pyo3(text_signature = "(htmls, fragments)")]
    fn inline_many_fragments(
        &self,
        htmls: &Bound<'_, PyList>,
        css: &Bound<'_, PyList>,
    ) -> PyResult<Vec<String>> {
        inline_many_fragments_impl(&self.inner, htmls, css)
    }
}

/// inline(html, inline_style_tags=True, keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, cache=None, extra_css=None, preallocate_node_capacity=32)
///
/// Inline CSS in the given HTML document
#[pyfunction]
#[pyo3(
    signature = (html, inline_style_tags=true, keep_style_tags=false, keep_link_tags=false, base_url=None, load_remote_stylesheets=true, cache=None, extra_css=None, preallocate_node_capacity=32)
)]
#[allow(clippy::too_many_arguments)]
fn inline(
    html: &str,
    inline_style_tags: Option<bool>,
    keep_style_tags: Option<bool>,
    keep_link_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
    cache: Option<StylesheetCache>,
    extra_css: Option<&str>,
    preallocate_node_capacity: Option<usize>,
) -> PyResult<String> {
    let inliner = inliner!(
        inline_style_tags,
        keep_style_tags,
        keep_link_tags,
        base_url,
        load_remote_stylesheets,
        cache,
        extra_css,
        preallocate_node_capacity
    );
    Ok(inliner.inline(html).map_err(InlineErrorWrapper)?)
}

/// inline_fragment(html, css, inline_style_tags=True, keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, cache=None, extra_css=None, preallocate_node_capacity=32)
///
/// Inline CSS in the given HTML fragment
#[pyfunction]
#[pyo3(
    signature = (html, css, inline_style_tags=true, keep_style_tags=false, keep_link_tags=false, base_url=None, load_remote_stylesheets=true, cache=None, extra_css=None, preallocate_node_capacity=32)
)]
#[allow(clippy::too_many_arguments)]
fn inline_fragment(
    html: &str,
    css: &str,
    inline_style_tags: Option<bool>,
    keep_style_tags: Option<bool>,
    keep_link_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
    cache: Option<StylesheetCache>,
    extra_css: Option<&str>,
    preallocate_node_capacity: Option<usize>,
) -> PyResult<String> {
    let inliner = inliner!(
        inline_style_tags,
        keep_style_tags,
        keep_link_tags,
        base_url,
        load_remote_stylesheets,
        cache,
        extra_css,
        preallocate_node_capacity
    );
    Ok(inliner
        .inline_fragment(html, css)
        .map_err(InlineErrorWrapper)?)
}

/// inline_many(htmls, inline_style_tags=True, keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, cache=None, extra_css=None, preallocate_node_capacity=32)
///
/// Inline CSS in multiple HTML documents
#[pyfunction]
#[pyo3(
    signature = (htmls, inline_style_tags=true, keep_style_tags=false, keep_link_tags=false, base_url=None, load_remote_stylesheets=true, cache=None, extra_css=None, preallocate_node_capacity=32)
)]
#[allow(clippy::too_many_arguments)]
fn inline_many(
    htmls: &Bound<'_, PyList>,
    inline_style_tags: Option<bool>,
    keep_style_tags: Option<bool>,
    keep_link_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
    cache: Option<StylesheetCache>,
    extra_css: Option<&str>,
    preallocate_node_capacity: Option<usize>,
) -> PyResult<Vec<String>> {
    let inliner = inliner!(
        inline_style_tags,
        keep_style_tags,
        keep_link_tags,
        base_url,
        load_remote_stylesheets,
        cache,
        extra_css,
        preallocate_node_capacity
    );
    inline_many_impl(&inliner, htmls)
}

fn inline_many_impl(
    inliner: &rust_inline::CSSInliner<'_>,
    htmls: &Bound<'_, PyList>,
) -> PyResult<Vec<String>> {
    // Extract strings from the list. It will fail if there is any non-string value
    let extracted: Result<Vec<_>, _> = htmls.iter().map(|h| h.extract::<String>()).collect();
    let output: Result<Vec<_>, _> = extracted?
        .par_iter()
        .map(|html| inliner.inline(html))
        .collect();
    Ok(output.map_err(InlineErrorWrapper)?)
}

/// inline_many_fragments(htmls, css, inline_style_tags=True, keep_style_tags=False, keep_link_tags=False, base_url=None, load_remote_stylesheets=True, cache=None, extra_css=None, preallocate_node_capacity=32)
///
/// Inline CSS in multiple HTML fragments
#[pyfunction]
#[pyo3(
    signature = (htmls, css, inline_style_tags=true, keep_style_tags=false, keep_link_tags=false, base_url=None, load_remote_stylesheets=true, cache=None, extra_css=None, preallocate_node_capacity=32)
)]
#[allow(clippy::too_many_arguments)]
fn inline_many_fragments(
    htmls: &Bound<'_, PyList>,
    css: &Bound<'_, PyList>,
    inline_style_tags: Option<bool>,
    keep_style_tags: Option<bool>,
    keep_link_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
    cache: Option<StylesheetCache>,
    extra_css: Option<&str>,
    preallocate_node_capacity: Option<usize>,
) -> PyResult<Vec<String>> {
    let inliner = inliner!(
        inline_style_tags,
        keep_style_tags,
        keep_link_tags,
        base_url,
        load_remote_stylesheets,
        cache,
        extra_css,
        preallocate_node_capacity
    );
    inline_many_fragments_impl(&inliner, htmls, css)
}

fn inline_many_fragments_impl(
    inliner: &rust_inline::CSSInliner<'_>,
    htmls: &Bound<'_, PyList>,
    css: &Bound<'_, PyList>,
) -> PyResult<Vec<String>> {
    // Extract strings from the list. It will fail if there is any non-string value
    let extracted: Result<Vec<_>, _> = htmls.iter().map(|h| h.extract::<String>()).collect();
    let css: Result<Vec<_>, _> = css.iter().map(|c| c.extract::<String>()).collect();
    let output: Result<Vec<_>, _> = extracted?
        .par_iter()
        .zip(css?)
        .map(|(html, css)| inliner.inline_fragment(html, &css))
        .collect();
    Ok(output.map_err(InlineErrorWrapper)?)
}

#[allow(dead_code)]
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// Fast CSS inlining written in Rust
#[pymodule]
fn css_inline(py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<CSSInliner>()?;
    module.add_class::<StylesheetCache>()?;
    module.add_wrapped(wrap_pyfunction!(inline))?;
    module.add_wrapped(wrap_pyfunction!(inline_fragment))?;
    module.add_wrapped(wrap_pyfunction!(inline_many))?;
    module.add_wrapped(wrap_pyfunction!(inline_many_fragments))?;
    let inline_error = py.get_type_bound::<InlineError>();
    inline_error.setattr("__doc__", INLINE_ERROR_DOCSTRING)?;
    module.add("InlineError", inline_error)?;
    module.add("__build__", pyo3_built::pyo3_built!(py, build))?;
    Ok(())
}
