use css_inline as rust_inline;
use wasm_bindgen::prelude::*;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::convert::TryInto;

struct InlineErrorWrapper(rust_inline::InlineError);

impl From<InlineErrorWrapper> for JsValue {
    fn from(error: InlineErrorWrapper) -> Self {
        JsValue::from_str(error.0.to_string().as_str())
    }
}

struct UrlError(url::ParseError);

impl From<UrlError> for JsValue {
    fn from(error: UrlError) -> Self {
        wasm_bindgen::throw_str(&error.0.to_string())
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
pub struct Options {
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

#[wasm_bindgen]
pub fn inline(
    html: &str,
    options: JsValue,
) -> Result<String, JsValue> {
    let options: Options = if !options.is_undefined() {
        options.into_serde().map_err(SerdeError)?
    } else {
        Options::default()
    };
    let inliner = rust_inline::CSSInliner::new(options.try_into()?);
    Ok(inliner.inline(html).map_err(InlineErrorWrapper)?)
}
