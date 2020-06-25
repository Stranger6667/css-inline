use css_inline as rust_inline;
use pyo3::{create_exception, exceptions, prelude::*, types::PyList, wrap_pyfunction};
use rayon::prelude::*;

const MODULE_DOCSTRING: &str = "Fast CSS inlining written in Rust";
const INLINE_ERROR_DOCSTRING: &str = "An error that can occur during CSS inlining";

create_exception!(css_inline, InlineError, exceptions::ValueError);

fn to_pyerr(error: rust_inline::InlineError) -> PyErr {
    match error {
        rust_inline::InlineError::IO(error) => InlineError::py_err(format!("{}", error)),
        rust_inline::InlineError::Network(error) => InlineError::py_err(format!("{}", error)),
        rust_inline::InlineError::ParseError(message) => InlineError::py_err(message),
    }
}

struct UrlError(url::ParseError);

impl From<UrlError> for PyErr {
    fn from(error: UrlError) -> Self {
        exceptions::ValueError::py_err(format!("{}", error.0))
    }
}

fn parse_url(url: Option<String>) -> PyResult<Option<url::Url>> {
    Ok(if let Some(url) = url {
        Some(url::Url::parse(url.as_str()).map_err(UrlError)?)
    } else {
        None
    })
}

/// Customizable CSS inliner.
#[pyclass]
#[text_signature = "(remove_style_tags=False, base_url=None, load_remote_stylesheets=True)"]
struct CSSInliner {
    inner: rust_inline::CSSInliner,
}

#[pymethods]
impl CSSInliner {
    #[new]
    fn new(
        remove_style_tags: Option<bool>,
        base_url: Option<String>,
        load_remote_stylesheets: Option<bool>,
    ) -> PyResult<Self> {
        let options = rust_inline::InlineOptions {
            remove_style_tags: remove_style_tags.unwrap_or(false),
            base_url: parse_url(base_url)?,
            load_remote_stylesheets: load_remote_stylesheets.unwrap_or(true),
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
        Ok(self.inner.inline(html).map_err(to_pyerr)?)
    }

    /// inline_many(htmls)
    ///
    /// Inline CSS in multiple HTML documents
    #[text_signature = "(htmls)"]
    fn inline_many(&self, htmls: &PyList) -> PyResult<Vec<String>> {
        inline_many_impl(&self.inner, htmls)
    }
}

/// inline(html, remove_style_tags=False, base_url=None, load_remote_stylesheets=True)
///
/// Inline CSS in the given HTML document
#[pyfunction]
#[text_signature = "(html, remove_style_tags=False, base_url=None, load_remote_stylesheets=True)"]
fn inline(
    html: &str,
    remove_style_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
) -> PyResult<String> {
    let options = rust_inline::InlineOptions {
        remove_style_tags: remove_style_tags.unwrap_or(false),
        base_url: parse_url(base_url)?,
        load_remote_stylesheets: load_remote_stylesheets.unwrap_or(true),
    };
    let inliner = rust_inline::CSSInliner::new(options);
    Ok(inliner.inline(html).map_err(to_pyerr)?)
}

/// inline_many(htmls, remove_style_tags=False, base_url=None, load_remote_stylesheets=True)
///
/// Inline CSS in multiple HTML documents
#[pyfunction]
#[text_signature = "(htmls, remove_style_tags=False, base_url=None, load_remote_stylesheets=True)"]
fn inline_many(
    htmls: &PyList,
    remove_style_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
) -> PyResult<Vec<String>> {
    let options = rust_inline::InlineOptions {
        remove_style_tags: remove_style_tags.unwrap_or(false),
        base_url: parse_url(base_url)?,
        load_remote_stylesheets: load_remote_stylesheets.unwrap_or(true),
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
    Ok(inlined.map_err(to_pyerr)?)
}

#[allow(dead_code)]
mod build {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[pymodule]
fn css_inline(py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<CSSInliner>()?;
    module.add_wrapped(wrap_pyfunction!(inline))?;
    module.add_wrapped(wrap_pyfunction!(inline_many))?;
    let inline_error = py.get_type::<InlineError>();
    inline_error.setattr("__doc__", INLINE_ERROR_DOCSTRING)?;
    module.add("InlineError", inline_error)?;
    module.add("__doc__", MODULE_DOCSTRING)?;
    module.add("__build__", pyo3_built::pyo3_built!(py, build))?;
    Ok(())
}
