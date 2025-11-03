use std::{fmt::Display, num::NonZeroUsize, sync::Mutex};

use ext_php_rs::{exception::PhpException, prelude::*, zend::ce};

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
        inline_style_tags = true,
        keep_style_tags = false,
        keep_link_tags = false,
        load_remote_stylesheets = true,
        base_url = None,
        extra_css = None,
        preallocate_node_capacity = 32_usize,
        cache = None,
    ))]
    #[php(optional = inline_style_tags)]
    pub fn __construct(
        inline_style_tags: bool,
        keep_style_tags: bool,
        keep_link_tags: bool,
        load_remote_stylesheets: bool,
        base_url: Option<String>,
        extra_css: Option<String>,
        preallocate_node_capacity: usize,
        cache: Option<&StylesheetCache>,
    ) -> PhpResult<CssInliner> {
        let base_url = if let Some(url) = base_url {
            Some(css_inline::Url::parse(&url).map_err(from_error)?)
        } else {
            None
        };

        let cache = if let Some(cache) = cache {
            Some(Mutex::new(css_inline::StylesheetCache::new(cache.size)))
        } else {
            None
        };

        let options = css_inline::InlineOptions {
            inline_style_tags,
            keep_style_tags,
            keep_link_tags,
            base_url,
            load_remote_stylesheets,
            extra_css: extra_css.map(Into::into),
            preallocate_node_capacity,
            cache,
            ..Default::default()
        };

        Ok(CssInliner {
            inner: css_inline::CSSInliner::new(options),
        })
    }

    pub fn inline(&self, html: &str) -> PhpResult<String> {
        self.inner.inline(html).map_err(from_error)
    }

    pub fn inline_fragment(&self, html: &str, css: &str) -> PhpResult<String> {
        self.inner.inline_fragment(html, css).map_err(from_error)
    }
}

#[php_function]
#[php(name = "CssInline\\inline")]
pub fn inline(html: &str) -> PhpResult<String> {
    css_inline::inline(html).map_err(from_error)
}

#[php_function]
#[php(name = "CssInline\\inline_fragment")]
pub fn inline_fragment(fragment: &str, css: &str) -> PhpResult<String> {
    css_inline::inline_fragment(fragment, css).map_err(from_error)
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<InlineError>()
        .class::<StylesheetCache>()
        .class::<CssInliner>()
        .function(wrap_function!(inline))
        .function(wrap_function!(inline_fragment))
}
