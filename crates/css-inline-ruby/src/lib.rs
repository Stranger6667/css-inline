//! Ruby bindings for css-inline
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
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences,
    rust_2018_idioms,
    rust_2018_compatibility,
    rust_2021_compatibility
)]
use css_inline as rust_inline;
use magnus::{
    DataTypeFunctions, RHash, Ruby, TryConvert, TypedData, Value, function, method,
    prelude::*,
    scan_args::{Args, get_kwargs, scan_args},
    typed_data::Obj,
};
use rayon::prelude::*;
use std::{
    borrow::Cow,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

type RubyResult<T> = Result<T, magnus::Error>;

#[allow(clippy::struct_excessive_bools)]
struct Options {
    /// Whether to inline CSS from "style" tags.
    ///
    /// Sometimes HTML may include a lot of boilerplate styles, that are not applicable in every
    /// scenario, and it is useful to ignore them and use `extra_css` instead.
    inline_style_tags: Option<bool>,
    /// Keep "style" tags after inlining.
    keep_style_tags: Option<bool>,
    /// Keep "link" tags after inlining.
    keep_link_tags: Option<bool>,
    /// Keep "at-rules" after inlining.
    keep_at_rules: Option<bool>,
    /// Remove trailing semicolons and spaces between properties and values.
    minify_css: Option<bool>,
    /// Used for loading external stylesheets via relative URLs.
    base_url: Option<String>,
    /// Whether remote stylesheets should be loaded or not.
    load_remote_stylesheets: Option<bool>,
    /// An LRU Cache for external stylesheets.
    cache: Option<Obj<StylesheetCache>>,
    /// Additional CSS to inline.
    extra_css: Option<String>,
    /// Pre-allocate capacity for HTML nodes during parsing.
    /// It can improve performance when you have an estimate of the number of nodes in your HTML document.
    preallocate_node_capacity: Option<usize>,
    /// Remove selectors that were successfully inlined from inline `<style>` blocks.
    remove_inlined_selectors: Option<bool>,
    /// Apply `width` HTML attributes from CSS `width` properties on supported elements.
    apply_width_attributes: Option<bool>,
    /// Apply `height` HTML attributes from CSS `height` properties on supported elements.
    apply_height_attributes: Option<bool>,
}

impl TryConvert for Options {
    fn try_convert(v: Value) -> Result<Self, magnus::Error> {
        let h = RHash::try_convert(v)?;
        let ruby = Ruby::get_with(v);
        Ok(Self {
            inline_style_tags: h.aref::<_, Option<bool>>(ruby.to_symbol("inline_style_tags"))?,
            keep_style_tags: h.aref::<_, Option<bool>>(ruby.to_symbol("keep_style_tags"))?,
            keep_link_tags: h.aref::<_, Option<bool>>(ruby.to_symbol("keep_link_tags"))?,
            keep_at_rules: h.aref::<_, Option<bool>>(ruby.to_symbol("keep_at_rules"))?,
            minify_css: h.aref::<_, Option<bool>>(ruby.to_symbol("minify_css"))?,
            base_url: h.aref::<_, Option<String>>(ruby.to_symbol("base_url"))?,
            load_remote_stylesheets: h
                .aref::<_, Option<bool>>(ruby.to_symbol("load_remote_stylesheets"))?,
            cache: h.aref::<_, Option<Obj<StylesheetCache>>>(ruby.to_symbol("cache"))?,
            extra_css: h.aref::<_, Option<String>>(ruby.to_symbol("extra_css"))?,
            preallocate_node_capacity: h
                .aref::<_, Option<usize>>(ruby.to_symbol("preallocate_node_capacity"))?,
            remove_inlined_selectors: h
                .aref::<_, Option<bool>>(ruby.to_symbol("remove_inlined_selectors"))?,
            apply_width_attributes: h
                .aref::<_, Option<bool>>(ruby.to_symbol("apply_width_attributes"))?,
            apply_height_attributes: h
                .aref::<_, Option<bool>>(ruby.to_symbol("apply_height_attributes"))?,
        })
    }
}

fn parse_options<Req>(
    args: &Args<Req, (), (), (), RHash, ()>,
) -> RubyResult<rust_inline::InlineOptions<'static>> {
    let kwargs: Options = Options::try_convert(args.keywords.as_value())?;
    Ok(rust_inline::InlineOptions {
        inline_style_tags: kwargs.inline_style_tags.unwrap_or(true),
        keep_style_tags: kwargs.keep_style_tags.unwrap_or(false),
        keep_link_tags: kwargs.keep_link_tags.unwrap_or(false),
        keep_at_rules: kwargs.keep_at_rules.unwrap_or(false),
        minify_css: kwargs.minify_css.unwrap_or(false),
        base_url: parse_url(kwargs.base_url)?,
        load_remote_stylesheets: kwargs.load_remote_stylesheets.unwrap_or(true),
        cache: kwargs
            .cache
            .map(|cache| Mutex::new(rust_inline::StylesheetCache::new(cache.size))),
        extra_css: kwargs.extra_css.map(Cow::Owned),
        preallocate_node_capacity: kwargs.preallocate_node_capacity.unwrap_or(32),
        resolver: Arc::new(rust_inline::DefaultStylesheetResolver),
        remove_inlined_selectors: kwargs.remove_inlined_selectors.unwrap_or(false),
        apply_width_attributes: kwargs.apply_width_attributes.unwrap_or(false),
        apply_height_attributes: kwargs.apply_height_attributes.unwrap_or(false),
    })
}

#[derive(DataTypeFunctions, TypedData)]
#[magnus(class = "CSSInline::StylesheetCache")]
struct StylesheetCache {
    size: NonZeroUsize,
}

impl StylesheetCache {
    fn new(args: &[Value]) -> RubyResult<StylesheetCache> {
        fn error() -> magnus::Error {
            let ruby = Ruby::get().expect("Always called from a Ruby thread");
            magnus::Error::new(
                ruby.exception_arg_error(),
                "Cache size must be an integer greater than zero",
            )
        }

        let args: Args<(), (), (), (), RHash, ()> = scan_args::<(), _, _, _, RHash, _>(args)?;
        let kwargs = get_kwargs::<_, (), (Option<usize>,), ()>(args.keywords, &[], &["size"])
            .map_err(|_| error())?;
        let size = NonZeroUsize::new(kwargs.optional.0.unwrap_or(8)).ok_or_else(error)?;
        Ok(StylesheetCache { size })
    }
}

#[magnus::wrap(class = "CSSInline::CSSInliner")]
struct CSSInliner {
    inner: rust_inline::CSSInliner<'static>,
}

struct InlineErrorWrapper(rust_inline::InlineError);

impl From<InlineErrorWrapper> for magnus::Error {
    fn from(error: InlineErrorWrapper) -> Self {
        let ruby = Ruby::get().expect("Always called from a Ruby thread");
        match error.0 {
            rust_inline::InlineError::IO(error) => {
                magnus::Error::new(ruby.exception_arg_error(), error.to_string())
            }
            rust_inline::InlineError::Network { error, location } => {
                magnus::Error::new(ruby.exception_arg_error(), format!("{error}: {location}"))
            }
            rust_inline::InlineError::ParseError(message) => {
                magnus::Error::new(ruby.exception_arg_error(), message.to_string())
            }
            rust_inline::InlineError::MissingStyleSheet { .. } => {
                magnus::Error::new(ruby.exception_arg_error(), error.0.to_string())
            }
        }
    }
}

struct UrlError {
    error: rust_inline::ParseError,
    url: String,
}

impl From<UrlError> for magnus::Error {
    fn from(error: UrlError) -> magnus::Error {
        let ruby = Ruby::get().expect("Always called from a Ruby thread");
        magnus::Error::new(
            ruby.exception_arg_error(),
            format!("{}: {}", error.error, error.url),
        )
    }
}

fn parse_url(url: Option<String>) -> RubyResult<Option<rust_inline::Url>> {
    Ok(if let Some(url) = url {
        Some(rust_inline::Url::parse(url.as_str()).map_err(|error| UrlError { error, url })?)
    } else {
        None
    })
}

impl CSSInliner {
    fn new(args: &[Value]) -> RubyResult<CSSInliner> {
        let args = scan_args::<(), _, _, _, _, _>(args)?;
        let options = parse_options(&args)?;
        Ok(CSSInliner {
            inner: rust_inline::CSSInliner::new(options),
        })
    }

    #[allow(clippy::needless_pass_by_value)]
    fn inline(&self, html: String) -> RubyResult<String> {
        Ok(self.inner.inline(&html).map_err(InlineErrorWrapper)?)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn inline_fragment(&self, html: String, css: String) -> RubyResult<String> {
        Ok(self
            .inner
            .inline_fragment(&html, &css)
            .map_err(InlineErrorWrapper)?)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn inline_many(&self, html: Vec<String>) -> RubyResult<Vec<String>> {
        inline_many_impl(&html, &self.inner)
    }

    #[allow(clippy::needless_pass_by_value)]
    fn inline_many_fragments(
        &self,
        html: Vec<String>,
        css: Vec<String>,
    ) -> RubyResult<Vec<String>> {
        inline_many_fragments_impl(&html, &css, &self.inner)
    }
}

fn inline(args: &[Value]) -> RubyResult<String> {
    let args = scan_args::<(String,), _, _, _, _, _>(args)?;
    let options = parse_options(&args)?;
    let html = args.required.0;
    Ok(rust_inline::CSSInliner::new(options)
        .inline(&html)
        .map_err(InlineErrorWrapper)?)
}

fn inline_fragment(args: &[Value]) -> RubyResult<String> {
    let args = scan_args::<(String, String), _, _, _, _, _>(args)?;
    let options = parse_options(&args)?;
    let html = args.required.0;
    let css = args.required.1;
    Ok(rust_inline::CSSInliner::new(options)
        .inline_fragment(&html, &css)
        .map_err(InlineErrorWrapper)?)
}

fn inline_many(args: &[Value]) -> RubyResult<Vec<String>> {
    let args = scan_args::<(Vec<String>,), _, _, _, _, _>(args)?;
    let options = parse_options(&args)?;
    let inliner = rust_inline::CSSInliner::new(options);
    inline_many_impl(&args.required.0, &inliner)
}

fn inline_many_impl(
    htmls: &[String],
    inliner: &rust_inline::CSSInliner<'static>,
) -> RubyResult<Vec<String>> {
    let output: Result<Vec<_>, _> = htmls.par_iter().map(|html| inliner.inline(html)).collect();
    Ok(output.map_err(InlineErrorWrapper)?)
}

fn inline_many_fragments(args: &[Value]) -> RubyResult<Vec<String>> {
    let args = scan_args::<(Vec<String>, Vec<String>), _, _, _, _, _>(args)?;
    let options = parse_options(&args)?;
    let inliner = rust_inline::CSSInliner::new(options);
    inline_many_fragments_impl(&args.required.0, &args.required.1, &inliner)
}

fn inline_many_fragments_impl(
    htmls: &[String],
    css: &[String],
    inliner: &rust_inline::CSSInliner<'static>,
) -> RubyResult<Vec<String>> {
    let output: Result<Vec<_>, _> = htmls
        .par_iter()
        .zip(css)
        .map(|(html, css)| inliner.inline_fragment(html, css))
        .collect();
    Ok(output.map_err(InlineErrorWrapper)?)
}

#[magnus::init(name = "css_inline")]
fn init(ruby: &Ruby) -> RubyResult<()> {
    let module = ruby.define_module("CSSInline")?;

    module.define_module_function("inline", function!(inline, -1))?;
    module.define_module_function("inline_fragment", function!(inline_fragment, -1))?;
    module.define_module_function("inline_many", function!(inline_many, -1))?;
    module.define_module_function(
        "inline_many_fragments",
        function!(inline_many_fragments, -1),
    )?;

    let class = module.define_class("CSSInliner", ruby.class_object())?;
    class.define_singleton_method("new", function!(CSSInliner::new, -1))?;
    class.define_method("inline", method!(CSSInliner::inline, 1))?;
    class.define_method("inline_fragment", method!(CSSInliner::inline_fragment, 2))?;
    class.define_method("inline_many", method!(CSSInliner::inline_many, 1))?;
    class.define_method(
        "inline_many_fragments",
        method!(CSSInliner::inline_many_fragments, 2),
    )?;

    let class = module.define_class("StylesheetCache", ruby.class_object())?;
    class.define_singleton_method("new", function!(StylesheetCache::new, -1))?;
    Ok(())
}
