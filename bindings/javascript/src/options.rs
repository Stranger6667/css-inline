use crate::errors::{JsError, UrlError};
#[cfg(not(target_arch = "wasm32"))]
use napi_derive::napi;
use std::{borrow::Cow, sync::Arc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::JsValue;

fn parse_url(url: Option<String>) -> std::result::Result<Option<css_inline::Url>, JsError> {
    Ok(if let Some(url) = url {
        Some(css_inline::Url::parse(url.as_str()).map_err(|error| UrlError { error, url })?)
    } else {
        None
    })
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(not(target_arch = "wasm32"), napi(object))]
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default)]
pub struct StylesheetCache {
    /// Cache size.
    pub size: u32,
}

#[cfg_attr(
    target_arch = "wasm32",
    derive(serde::Deserialize),
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[cfg_attr(not(target_arch = "wasm32"), napi(object))]
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Default)]
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
    /// Keep "at-rules" after inlining.
    pub keep_at_rules: Option<bool>,
    /// Remove trailing semicolons and spaces between properties and values.
    pub minify_css: Option<bool>,
    /// Used for loading external stylesheets via relative URLs.
    pub base_url: Option<String>,
    /// Whether remote stylesheets should be loaded or not.
    pub load_remote_stylesheets: Option<bool>,
    #[cfg(not(target_arch = "wasm32"))]
    /// An LRU Cache for external stylesheets.
    pub cache: Option<StylesheetCache>,
    /// Additional CSS to inline.
    pub extra_css: Option<String>,
    /// Pre-allocate capacity for HTML nodes during parsing.
    /// It can improve performance when you have an estimate of the number of nodes in your HTML document.
    pub preallocate_node_capacity: Option<u32>,
    /// Remove selectors that were successfully inlined from inline `<style>` blocks.
    pub remove_inlined_selectors: Option<bool>,
}

impl TryFrom<Options> for css_inline::InlineOptions<'_> {
    type Error = JsError;

    fn try_from(value: Options) -> std::result::Result<Self, Self::Error> {
        Ok(css_inline::InlineOptions {
            inline_style_tags: value.inline_style_tags.unwrap_or(true),
            keep_style_tags: value.keep_style_tags.unwrap_or(false),
            keep_link_tags: value.keep_link_tags.unwrap_or(false),
            keep_at_rules: value.keep_at_rules.unwrap_or(false),
            minify_css: value.minify_css.unwrap_or(false),
            base_url: parse_url(value.base_url)?,
            load_remote_stylesheets: value.load_remote_stylesheets.unwrap_or(true),
            extra_css: value.extra_css.map(Cow::Owned),
            #[cfg(not(target_arch = "wasm32"))]
            cache: {
                if let Some(cache) = value.cache {
                    let size =
                        std::num::NonZeroUsize::new(cache.size as usize).ok_or_else(|| {
                            let reason =
                                "Cache size must be an integer greater than zero".to_string();
                            napi::Error::from_reason(reason)
                        })?;
                    Some(std::sync::Mutex::new(css_inline::StylesheetCache::new(
                        size,
                    )))
                } else {
                    None
                }
            },
            preallocate_node_capacity: if let Some(capacity) = value.preallocate_node_capacity {
                usize::try_from(capacity).map_err(|_| {
                    let reason = "Invalid capacity".to_string();
                    #[cfg(target_arch = "wasm32")]
                    {
                        JsValue::from_str(reason.as_str())
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        napi::Error::from_reason(reason)
                    }
                })?
            } else {
                32
            },
            resolver: {
                #[cfg(target_arch = "wasm32")]
                {
                    #[derive(Debug, Default)]
                    pub struct UnsupportedResolver;

                    impl css_inline::StylesheetResolver for UnsupportedResolver {
                        fn retrieve(&self, location: &str) -> css_inline::Result<String> {
                            let message = if location.starts_with("https")
                                || location.starts_with("http")
                            {
                                format!(
                                    "Loading remote stylesheets is not supported on WASM: {location}"
                                )
                            } else {
                                format!("Loading local files is not supported on WASM: {location}")
                            };
                            Err(self.unsupported(&message))
                        }
                    }
                    Arc::new(UnsupportedResolver)
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Arc::new(css_inline::DefaultStylesheetResolver)
                }
            },
            remove_inlined_selectors: value.remove_inlined_selectors.unwrap_or(false),
        })
    }
}
