#![allow(non_snake_case)]

use std::{fmt::Display, num::NonZeroUsize, sync::Mutex};

use ext_php_rs::{exception::PhpException, prelude::*, zend::ce};
use rayon::prelude::*;

#[php_const]
#[php(name = "CssInline\\VERSION")]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[php_class]
#[php(name = "CssInline\\InlineError")]
#[php(extends(ce = ce::exception, stub = "\\Exception"))]
#[derive(Default)]
pub struct InlineError;

fn from_error<E: Display>(error: E) -> PhpException {
    PhpException::from_class::<InlineError>(error.to_string())
}

#[php_class]
#[php(name = "CssInline\\StylesheetCache")]
pub struct StylesheetCache {
    size: NonZeroUsize,
}

#[php_impl]
impl StylesheetCache {
    pub fn __construct(size: usize) -> PhpResult<StylesheetCache> {
        let size = NonZeroUsize::new(size).ok_or_else(|| {
            PhpException::default("Cache size must be an integer greater than zero".to_string())
        })?;
        Ok(StylesheetCache { size })
    }
}

#[php_class]
#[php(name = "CssInline\\CssInliner")]
pub struct CssInliner {
    inner: css_inline::CSSInliner<'static>,
}

#[php_impl]
impl CssInliner {
    #[php(defaults(
        inlineStyleTags = true,
        keepStyleTags = false,
        keepLinkTags = false,
        keepAtRules = false,
        minifyCss = false,
        loadRemoteStylesheets = true,
        baseUrl = None,
        extraCss = None,
        preallocateNodeCapacity = 32,
        cache = None,
        removeInlinedSelectors = false,
    ))]
    #[php(optional = inlineStyleTags)]
    #[allow(clippy::too_many_arguments)]
    pub fn __construct(
        inlineStyleTags: bool,
        keepStyleTags: bool,
        keepLinkTags: bool,
        keepAtRules: bool,
        minifyCss: bool,
        loadRemoteStylesheets: bool,
        baseUrl: Option<String>,
        extraCss: Option<String>,
        preallocateNodeCapacity: i64,
        cache: Option<&StylesheetCache>,
        removeInlinedSelectors: bool,
    ) -> PhpResult<CssInliner> {
        if preallocateNodeCapacity < 0 {
            return Err(PhpException::default(
                "preallocateNodeCapacity must be a non-negative integer".to_string(),
            ));
        }

        let base_url = if let Some(url) = baseUrl {
            Some(css_inline::Url::parse(&url).map_err(from_error)?)
        } else {
            None
        };

        let stylesheet_cache =
            cache.map(|cache| Mutex::new(css_inline::StylesheetCache::new(cache.size)));

        #[allow(clippy::cast_sign_loss)]
        let options = css_inline::InlineOptions {
            inline_style_tags: inlineStyleTags,
            keep_style_tags: keepStyleTags,
            keep_link_tags: keepLinkTags,
            keep_at_rules: keepAtRules,
            minify_css: minifyCss,
            base_url,
            load_remote_stylesheets: loadRemoteStylesheets,
            extra_css: extraCss.map(Into::into),
            preallocate_node_capacity: preallocateNodeCapacity as usize,
            cache: stylesheet_cache,
            remove_inlined_selectors: removeInlinedSelectors,
            ..Default::default()
        };

        Ok(CssInliner {
            inner: css_inline::CSSInliner::new(options),
        })
    }

    pub fn inline(&self, html: &str) -> PhpResult<String> {
        self.inner.inline(html).map_err(from_error)
    }

    #[php(name = "inlineFragment")]
    pub fn inline_fragment(&self, html: &str, css: &str) -> PhpResult<String> {
        self.inner.inline_fragment(html, css).map_err(from_error)
    }

    #[php(name = "inlineMany")]
    pub fn inline_many(&self, htmls: Vec<String>) -> PhpResult<Vec<String>> {
        htmls
            .par_iter()
            .map(|html| self.inner.inline(html))
            .collect::<Result<Vec<_>, _>>()
            .map_err(from_error)
    }

    #[php(name = "inlineManyFragments")]
    pub fn inline_many_fragments(&self, htmls: Vec<String>, css: &str) -> PhpResult<Vec<String>> {
        htmls
            .par_iter()
            .map(|html| self.inner.inline_fragment(html, css))
            .collect::<Result<Vec<_>, _>>()
            .map_err(from_error)
    }
}

#[php_function]
#[php(name = "CssInline\\inline")]
pub fn inline(html: &str) -> PhpResult<String> {
    css_inline::inline(html).map_err(from_error)
}

#[php_function]
#[php(name = "CssInline\\inlineFragment")]
pub fn inline_fragment(fragment: &str, css: &str) -> PhpResult<String> {
    css_inline::inline_fragment(fragment, css).map_err(from_error)
}

#[php_function]
#[php(name = "CssInline\\inlineMany")]
pub fn inline_many(htmls: Vec<String>) -> PhpResult<Vec<String>> {
    let inliner = css_inline::CSSInliner::default();
    htmls
        .par_iter()
        .map(|html| inliner.inline(html))
        .collect::<Result<Vec<_>, _>>()
        .map_err(from_error)
}

#[php_function]
#[php(name = "CssInline\\inlineManyFragments")]
pub fn inline_many_fragments(htmls: Vec<String>, css: &str) -> PhpResult<Vec<String>> {
    let inliner = css_inline::CSSInliner::default();
    htmls
        .par_iter()
        .map(|html| inliner.inline_fragment(html, css))
        .collect::<Result<Vec<_>, _>>()
        .map_err(from_error)
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .constant(wrap_constant!(VERSION))
        .class::<InlineError>()
        .class::<StylesheetCache>()
        .class::<CssInliner>()
        .function(wrap_function!(inline))
        .function(wrap_function!(inline_fragment))
        .function(wrap_function!(inline_many))
        .function(wrap_function!(inline_many_fragments))
}
