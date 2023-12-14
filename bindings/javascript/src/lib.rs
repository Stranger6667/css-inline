#![deny(clippy::all)]
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::borrow::Cow;

#[napi(object)]
#[allow(clippy::struct_excessive_bools)]
#[derive(Default)]
pub struct Options {
    /// Whether to inline CSS from "style" tags.
    ///
    /// Sometimes HTML may include a lot of boilerplate styles, that are not applicable in every
    /// scenario, and it is useful to ignore them and use `extra_css` instead.
    pub inline_style_tags: Option<bool>,
    /// Keep "style" tags after inlining.
    pub keep_style_tags: Option<bool>,
    /// Keep "link" tags after inlining.
    pub keep_link_tags: Option<bool>,
    /// Used for loading external stylesheets via relative URLs.
    pub base_url: Option<String>,
    /// Whether remote stylesheets should be loaded or not.
    pub load_remote_stylesheets: Option<bool>,
    /// Additional CSS to inline.
    pub extra_css: Option<String>,
    /// Pre-allocate capacity for HTML nodes during parsing.
    /// It can improve performance when you have an estimate of the number of nodes in your HTML document.
    pub preallocate_node_capacity: Option<u32>,
}

impl TryFrom<Options> for css_inline::InlineOptions<'_> {
    type Error = napi::Error;

    fn try_from(value: Options) -> Result<Self> {
        Ok(css_inline::InlineOptions {
            inline_style_tags: value.inline_style_tags.unwrap_or(true),
            keep_style_tags: value.keep_style_tags.unwrap_or(false),
            keep_link_tags: value.keep_link_tags.unwrap_or(false),
            base_url: parse_url(value.base_url)?,
            load_remote_stylesheets: value.load_remote_stylesheets.unwrap_or(true),
            extra_css: value.extra_css.map(Cow::Owned),
            preallocate_node_capacity: if let Some(capacity) = value.preallocate_node_capacity {
                usize::try_from(capacity)
                    .map_err(|_| napi::Error::from_reason("Invalid capacity".to_string()))?
            } else {
                32
            },
        })
    }
}

struct UrlError(css_inline::ParseError);

impl From<UrlError> for napi::Error {
    fn from(err: UrlError) -> Self {
        napi::Error::new(Status::InvalidArg, err.0.to_string())
    }
}

fn parse_url(url: Option<String>) -> Result<Option<css_inline::Url>> {
    Ok(if let Some(url) = url {
        Some(css_inline::Url::parse(url.as_str()).map_err(UrlError)?)
    } else {
        None
    })
}

struct InlineErrorWrapper(css_inline::InlineError);

impl From<InlineErrorWrapper> for napi::Error {
    fn from(err: InlineErrorWrapper) -> Self {
        napi::Error::from_reason(err.0.to_string())
    }
}

#[napi]
/// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a string.
pub fn inline(html: String, options: Option<Options>) -> Result<String> {
    let options = options.unwrap_or_default();
    let inliner = css_inline::CSSInliner::new(options.try_into()?);
    Ok(inliner.inline(&html).map_err(InlineErrorWrapper)?)
}
