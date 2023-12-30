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
    class, define_module, function, method,
    prelude::*,
    scan_args::{get_kwargs, scan_args, Args},
    RHash, Value,
};
use rayon::prelude::*;
use std::{borrow::Cow, sync::Arc};

type RubyResult<T> = Result<T, magnus::Error>;

fn parse_options<Req>(
    args: &Args<Req, (), (), (), RHash, ()>,
) -> RubyResult<rust_inline::InlineOptions<'static>> {
    let kwargs = get_kwargs::<
        _,
        (),
        (
            Option<bool>,
            Option<bool>,
            Option<bool>,
            Option<String>,
            Option<bool>,
            Option<String>,
            Option<usize>,
        ),
        (),
    >(
        args.keywords,
        &[],
        &[
            "inline_style_tags",
            "keep_style_tags",
            "keep_link_tags",
            "base_url",
            "load_remote_stylesheets",
            "extra_css",
            "preallocate_node_capacity",
        ],
    )?;
    let kwargs = kwargs.optional;
    Ok(rust_inline::InlineOptions {
        inline_style_tags: kwargs.0.unwrap_or(true),
        keep_style_tags: kwargs.1.unwrap_or(false),
        keep_link_tags: kwargs.2.unwrap_or(false),
        base_url: parse_url(kwargs.3)?,
        load_remote_stylesheets: kwargs.4.unwrap_or(true),
        extra_css: kwargs.5.map(Cow::Owned),
        preallocate_node_capacity: kwargs.6.unwrap_or(32),
        resolver: Arc::new(rust_inline::DefaultStylesheetResolver),
    })
}

#[magnus::wrap(class = "CSSInline::CSSInliner")]
struct CSSInliner {
    inner: rust_inline::CSSInliner<'static>,
}

struct InlineErrorWrapper(rust_inline::InlineError);

impl From<InlineErrorWrapper> for magnus::Error {
    fn from(error: InlineErrorWrapper) -> Self {
        match error.0 {
            rust_inline::InlineError::IO(error) => {
                magnus::Error::new(magnus::exception::arg_error(), error.to_string())
            }
            rust_inline::InlineError::Network { error, location } => magnus::Error::new(
                magnus::exception::arg_error(),
                format!("{error}: {location}"),
            ),
            rust_inline::InlineError::ParseError(message) => {
                magnus::Error::new(magnus::exception::arg_error(), message.to_string())
            }
            rust_inline::InlineError::MissingStyleSheet { .. } => {
                magnus::Error::new(magnus::exception::arg_error(), error.0.to_string())
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
        magnus::Error::new(
            magnus::exception::arg_error(),
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
    fn inline_many(&self, html: Vec<String>) -> RubyResult<Vec<String>> {
        inline_many_impl(&html, &self.inner)
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

#[magnus::init(name = "css_inline")]
fn init() -> RubyResult<()> {
    let module = define_module("CSSInline")?;

    module.define_module_function("inline", function!(inline, -1))?;
    module.define_module_function("inline_many", function!(inline_many, -1))?;

    let class = module.define_class("CSSInliner", class::object())?;
    class.define_singleton_method("new", function!(CSSInliner::new, -1))?;
    class.define_method("inline", method!(CSSInliner::inline, 1))?;
    class.define_method("inline_many", method!(CSSInliner::inline_many, 1))?;

    Ok(())
}
