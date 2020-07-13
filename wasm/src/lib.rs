//! WASM bindings for css-inline
#![warn(
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::print_stdout,
    clippy::integer_arithmetic,
    clippy::cast_possible_truncation,
    clippy::result_unwrap_used,
    clippy::result_map_unwrap_or_else,
    clippy::option_unwrap_used,
    clippy::option_map_unwrap_or_else,
    clippy::option_map_unwrap_or,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
use css_inline as rust_inline;
use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
};
use wasm_bindgen::prelude::*;

struct InlineErrorWrapper(rust_inline::InlineError);

impl From<InlineErrorWrapper> for JsValue {
    fn from(error: InlineErrorWrapper) -> Self {
        JsValue::from_str(error.0.to_string().as_str())
    }
}

struct UrlError(url::ParseError);

impl From<UrlError> for JsValue {
    fn from(error: UrlError) -> Self {
        JsValue::from_str(error.0.to_string().as_str())
    }
}

fn parse_url(url: Option<String>) -> Result<Option<url::Url>, JsValue> {
    Ok(if let Some(url) = url {
        Some(url::Url::parse(url.as_str()).map_err(UrlError)?)
    } else {
        None
    })
}

#[macro_use]
extern crate serde_derive;

#[derive(Deserialize)]
#[serde(default)]
struct Options {
    inline_style_tags: bool,
    remove_style_tags: bool,
    base_url: Option<String>,
    load_remote_stylesheets: bool,
    extra_css: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            inline_style_tags: true,
            remove_style_tags: false,
            base_url: None,
            load_remote_stylesheets: true,
            extra_css: None,
        }
    }
}

struct SerdeError(serde_json::Error);

impl From<SerdeError> for JsValue {
    fn from(error: SerdeError) -> Self {
        JsValue::from_str(error.0.to_string().as_str())
    }
}

impl TryFrom<Options> for rust_inline::InlineOptions<'_> {
    type Error = JsValue;

    fn try_from(value: Options) -> Result<Self, Self::Error> {
        Ok(rust_inline::InlineOptions {
            inline_style_tags: value.inline_style_tags,
            remove_style_tags: value.remove_style_tags,
            base_url: parse_url(value.base_url)?,
            load_remote_stylesheets: value.load_remote_stylesheets,
            extra_css: value.extra_css.map(Cow::Owned),
        })
    }
}

/// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a string.
#[wasm_bindgen(skip_typescript)]
pub fn inline(html: &str, options: &JsValue) -> Result<String, JsValue> {
    let options: Options = if !options.is_undefined() {
        options.into_serde().map_err(SerdeError)?
    } else {
        Options::default()
    };
    let inliner = rust_inline::CSSInliner::new(options.try_into()?);
    Ok(inliner.inline(html).map_err(InlineErrorWrapper)?)
}

#[wasm_bindgen(typescript_custom_section)]
const INLINE: &'static str = r#"
interface InlineOptions {
    inline_style_tags?: boolean,
    remove_style_tags?: boolean,
    base_url?: string,
    load_remote_stylesheets?: boolean,
    extra_css?: string,
}

export function inline(html: string, options?: InlineOptions): string;
"#;
