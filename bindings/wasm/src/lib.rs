//! WASM bindings for css-inline
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
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences,
    rust_2018_idioms,
    rust_2018_compatibility,
    rust_2021_compatibility
)]
#[allow(clippy::module_name_repetitions)]
use css_inline as rust_inline;
use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
};
use wasm_bindgen::prelude::*;

struct InlineErrorWrapper(rust_inline::InlineError);

impl From<InlineErrorWrapper> for JsValue {
    fn from(error: InlineErrorWrapper) -> Self {
        if let rust_inline::InlineError::ParseError(e) = error.0 {
            JsValue::from_str(&e)
        } else {
            JsValue::from_str(error.0.to_string().as_str())
        }
    }
}

struct UrlError(rust_inline::ParseError);

impl From<UrlError> for JsValue {
    fn from(error: UrlError) -> Self {
        JsValue::from_str(error.0.to_string().as_str())
    }
}

fn parse_url(url: Option<String>) -> Result<Option<rust_inline::Url>, JsValue> {
    Ok(if let Some(url) = url {
        Some(rust_inline::Url::parse(url.as_str()).map_err(UrlError)?)
    } else {
        None
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Options {
    keep_style_tags: bool,
    keep_link_tags: bool,
    base_url: Option<String>,
    load_remote_stylesheets: bool,
    extra_css: Option<String>,
    preallocate_node_capacity: Option<usize>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            keep_style_tags: false,
            keep_link_tags: false,
            base_url: None,
            load_remote_stylesheets: true,
            extra_css: None,
            preallocate_node_capacity: None,
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
            keep_style_tags: value.keep_style_tags,
            keep_link_tags: value.keep_link_tags,
            base_url: parse_url(value.base_url)?,
            load_remote_stylesheets: value.load_remote_stylesheets,
            extra_css: value.extra_css.map(Cow::Owned),
            preallocate_node_capacity: value.preallocate_node_capacity.unwrap_or(32),
        })
    }
}

/// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a string.
#[wasm_bindgen(skip_typescript)]
pub fn inline(html: &str, options: JsValue) -> Result<String, JsValue> {
    let options: Options = if options.is_undefined() {
        Options::default()
    } else {
        serde_wasm_bindgen::from_value(options)?
    };
    let inliner = rust_inline::CSSInliner::new(options.try_into()?);
    Ok(inliner.inline(html).map_err(InlineErrorWrapper)?)
}

#[wasm_bindgen(typescript_custom_section)]
const INLINE: &'static str = r#"
interface InlineOptions {
    keep_style_tags?: boolean,
    base_url?: string,
    load_remote_stylesheets?: boolean,
    extra_css?: string,
    preallocate_node_capacity?: number,
}

export function inline(html: string, options?: InlineOptions): string;
"#;

#[cfg(test)]
pub mod tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn default_config() {
        let result = inline(
            "<html><head><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>",
            JsValue::undefined(),
        )
        .expect("Inlines correctly");
        assert_eq!(
            result,
            "<html><head></head><body><h1 style=\"color: red;\">Test</h1></body></html>"
        );
    }

    #[wasm_bindgen_test]
    fn remove_style_tags() {
        let options = Options {
            keep_style_tags: true,
            ..Options::default()
        };
        let options = serde_wasm_bindgen::to_value(&options).expect("Valid value");
        let result = inline(
            "<html><head><style>h1 { color:red; }</style></head><body><h1>Test</h1></body></html>",
            options,
        )
        .expect("Inlines correctly");
        assert_eq!(result, "<html><head><style>h1 { color:red; }</style></head><body><h1 style=\"color: red;\">Test</h1></body></html>");
    }
}
