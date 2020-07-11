use css_inline as rust_inline;
use wasm_bindgen::prelude::*;
use std::borrow::Cow;

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

#[wasm_bindgen]
pub fn inline(
    html: &str,
    inline_style_tags: Option<bool>,
    remove_style_tags: Option<bool>,
    base_url: Option<String>,
    load_remote_stylesheets: Option<bool>,
    extra_css: Option<String>,
) -> Result<String, JsValue> {
    let options = rust_inline::InlineOptions {
        inline_style_tags: inline_style_tags.unwrap_or(true),
        remove_style_tags: remove_style_tags.unwrap_or(false),
        base_url: parse_url(base_url)?,
        load_remote_stylesheets: load_remote_stylesheets.unwrap_or(true),
        extra_css: extra_css.map(Cow::Owned),
    };
    let inliner = rust_inline::CSSInliner::new(options);
    Ok(inliner.inline(html).map_err(InlineErrorWrapper)?)
}
