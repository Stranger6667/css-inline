#![doc = include_str!("../README.md")]
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
    missing_docs,
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
pub mod error;
mod hasher;
mod html;
mod parser;

pub use error::InlineError;
use indexmap::IndexMap;
use std::{
    borrow::Cow,
    hash::BuildHasherDefault,
    io::{ErrorKind, Write},
};

use crate::html::ElementStyleMap;
use hasher::BuildNoHashHasher;
use html::Document;
pub use url::{ParseError, Url};

/// Configuration options for CSS inlining process.
#[derive(Debug)]
pub struct InlineOptions<'a> {
    /// Keep "style" tags after inlining.
    pub keep_style_tags: bool,
    /// Keep "link" tags after inlining.
    pub keep_link_tags: bool,
    /// Used for loading external stylesheets via relative URLs.
    pub base_url: Option<Url>,
    /// Whether remote stylesheets should be loaded or not.
    pub load_remote_stylesheets: bool,
    // The point of using `Cow` here is Python bindings, where it is problematic to pass a reference
    // without dealing with memory leaks & unsafe. With `Cow` we can use moved values as `String` in
    // Python wrapper for `CSSInliner` and `&str` in Rust & simple functions on the Python side
    /// Additional CSS to inline.
    pub extra_css: Option<Cow<'a, str>>,
    /// Pre-allocate capacity for HTML nodes during parsing.
    /// It can improve performance when you have an estimate of the number of nodes in your HTML document.
    pub preallocate_node_capacity: usize,
}

impl<'a> InlineOptions<'a> {
    /// Override whether "style" tags should be kept after processing.
    #[must_use]
    pub fn keep_style_tags(mut self, keep_style_tags: bool) -> Self {
        self.keep_style_tags = keep_style_tags;
        self
    }

    /// Override whether "link" tags should be kept after processing.
    #[must_use]
    pub fn keep_link_tags(mut self, keep_link_tags: bool) -> Self {
        self.keep_link_tags = keep_link_tags;
        self
    }

    /// Set base URL that will be used for loading external stylesheets via relative URLs.
    #[must_use]
    pub fn base_url(mut self, base_url: Option<Url>) -> Self {
        self.base_url = base_url;
        self
    }

    /// Override whether remote stylesheets should be loaded.
    #[must_use]
    pub fn load_remote_stylesheets(mut self, load_remote_stylesheets: bool) -> Self {
        self.load_remote_stylesheets = load_remote_stylesheets;
        self
    }

    /// Set additional CSS to inline.
    #[must_use]
    pub fn extra_css(mut self, extra_css: Option<Cow<'a, str>>) -> Self {
        self.extra_css = extra_css;
        self
    }

    /// Set the initial node capacity for HTML tree.
    #[must_use]
    pub fn preallocate_node_capacity(mut self, preallocate_node_capacity: usize) -> Self {
        self.preallocate_node_capacity = preallocate_node_capacity;
        self
    }

    /// Create a new `CSSInliner` instance from this options.
    #[must_use]
    pub const fn build(self) -> CSSInliner<'a> {
        CSSInliner::new(self)
    }
}

impl Default for InlineOptions<'_> {
    #[inline]
    fn default() -> Self {
        InlineOptions {
            keep_style_tags: false,
            keep_link_tags: false,
            base_url: None,
            load_remote_stylesheets: true,
            extra_css: None,
            preallocate_node_capacity: 32,
        }
    }
}

type Result<T> = std::result::Result<T, InlineError>;

/// Customizable CSS inliner.
#[derive(Debug)]
pub struct CSSInliner<'a> {
    options: InlineOptions<'a>,
}

const GROWTH_COEFFICIENT: f64 = 1.5;
// A rough coefficient to calculate the number of individual declarations based on the total CSS size.
const DECLARATION_SIZE_COEFFICIENT: f64 = 30.0;

impl<'a> CSSInliner<'a> {
    /// Create a new `CSSInliner` instance with given options.
    #[must_use]
    #[inline]
    pub const fn new(options: InlineOptions<'a>) -> Self {
        CSSInliner { options }
    }

    /// Return a default `InlineOptions` that can fully configure the CSS inliner.
    ///
    /// # Examples
    ///
    /// Get default `InlineOptions`, then change base url
    ///
    /// ```rust
    /// use css_inline::{CSSInliner, Url};
    /// # use url::ParseError;
    /// # fn run() -> Result<(), ParseError> {
    /// let url = Url::parse("https://api.example.com")?;
    /// let inliner = CSSInliner::options()
    ///     .base_url(Some(url))
    ///     .build();
    /// # Ok(())
    /// # }
    /// # run().unwrap();
    /// ```
    #[must_use]
    #[inline]
    pub fn options() -> InlineOptions<'a> {
        InlineOptions::default()
    }

    /// Inline CSS styles from <style> tags to matching elements in the HTML tree and return a
    /// string.
    ///
    /// # Errors
    ///
    /// Inlining might fail for the following reasons:
    ///   - Missing stylesheet file;
    ///   - Remote stylesheet is not available;
    ///   - IO errors;
    ///   - Internal CSS selector parsing error;
    #[inline]
    pub fn inline(&self, html: &str) -> Result<String> {
        // Allocating more memory than the input HTML, as the inlined version is usually bigger
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation
        )]
        let mut out = Vec::with_capacity(
            (html.len() as f64 * GROWTH_COEFFICIENT)
                .min(usize::MAX as f64)
                .round() as usize,
        );
        self.inline_to(html, &mut out)?;
        Ok(String::from_utf8_lossy(&out).to_string())
    }

    /// Inline CSS & write the result to a generic writer. Use it if you want to write
    /// the inlined document to a file.
    ///
    /// # Errors
    ///
    /// Inlining might fail for the following reasons:
    ///   - Missing stylesheet file;
    ///   - Remote stylesheet is not available;
    ///   - IO errors;
    ///   - Internal CSS selector parsing error;
    #[inline]
    pub fn inline_to<W: Write>(&self, html: &str, target: &mut W) -> Result<()> {
        let document =
            Document::parse_with_options(html.as_bytes(), self.options.preallocate_node_capacity);
        // CSS rules may overlap, and the final set of rules applied to an element depend on
        // selectors' specificity - selectors with higher specificity have more priority.
        // Inlining happens in two major steps:
        //   1. All available styles are mapped to respective elements together with their
        //      selector's specificity. When two rules overlap on the same declaration, then
        //      the one with higher specificity replaces another.
        //   2. Resulting styles are merged into existing "style" tags.
        let mut size_estimate: usize = document
            .styles()
            .map(|s| {
                // Add 1 to account for the extra `\n` char we add between styles
                s.len().saturating_add(1)
            })
            .sum();
        if let Some(extra_css) = &self.options.extra_css {
            size_estimate = size_estimate.saturating_add(extra_css.len());
        }
        let mut raw_styles = String::with_capacity(size_estimate);
        for style in document.styles() {
            raw_styles.push_str(style);
            raw_styles.push('\n');
        }
        if self.options.load_remote_stylesheets {
            let mut links = document.stylesheets().collect::<Vec<&str>>();
            links.sort_unstable();
            links.dedup();
            for href in &links {
                let url = self.get_full_url(href);
                let css = load_external(url.as_ref())?;
                raw_styles.push_str(&css);
                raw_styles.push('\n');
            }
        }
        if let Some(extra_css) = &self.options.extra_css {
            raw_styles.push_str(extra_css);
        }
        let mut styles = IndexMap::with_capacity_and_hasher(128, BuildNoHashHasher::default());
        let mut parse_input = cssparser::ParserInput::new(&raw_styles);
        let mut parser = cssparser::Parser::new(&mut parse_input);
        // Allocating some memory for all the parsed declarations
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation
        )]
        let mut declarations = Vec::with_capacity(
            ((raw_styles.len() as f64 / DECLARATION_SIZE_COEFFICIENT)
                .min(usize::MAX as f64)
                .round() as usize)
                .max(16),
        );
        let mut rule_list = Vec::with_capacity(declarations.capacity() / 3);
        for rule in cssparser::RuleListParser::new_for_stylesheet(
            &mut parser,
            parser::CSSRuleListParser::new(&mut declarations),
        )
        .flatten()
        {
            rule_list.push(rule);
        }
        for (selectors, (start, end)) in &rule_list {
            // Only CSS Syntax Level 3 is supported, therefore it is OK to split by `,`
            // With `is` or `where` selectors (Level 4) this split should be done on the parser level
            for selector in selectors.split(',') {
                if let Ok(matching_elements) = document.select(selector) {
                    let specificity = matching_elements.specificity();
                    for matching_element in matching_elements {
                        let element_styles =
                            styles.entry(matching_element.node_id).or_insert_with(|| {
                                ElementStyleMap::with_capacity_and_hasher(
                                    end.saturating_sub(*start).saturating_add(4),
                                    BuildHasherDefault::default(),
                                )
                            });
                        // Iterate over pairs of property name & value
                        // Example: `padding`, `0`
                        for (name, value) in &declarations[*start..*end] {
                            match element_styles.entry(name.as_ref()) {
                                indexmap::map::Entry::Occupied(mut entry) => {
                                    if entry.get().0 <= specificity {
                                        entry.insert((specificity, *value));
                                    }
                                }
                                indexmap::map::Entry::Vacant(entry) => {
                                    entry.insert((specificity, *value));
                                }
                            }
                        }
                    }
                }
                // Ignore not parsable selectors. E.g. there is no parser for @media queries
                // Which means that they will fall into this category and will be ignored
            }
        }
        document.serialize(
            target,
            styles,
            self.options.keep_style_tags,
            self.options.keep_link_tags,
        )?;
        Ok(())
    }

    fn get_full_url<'u>(&self, href: &'u str) -> Cow<'u, str> {
        // Valid absolute URL
        if Url::parse(href).is_ok() {
            return Cow::Borrowed(href);
        };
        if let Some(base_url) = &self.options.base_url {
            // Use the same scheme as the base URL
            if href.starts_with("//") {
                return Cow::Owned(format!("{}:{}", base_url.scheme(), href));
            }
            // Not a URL, then it is a relative URL
            if let Ok(new_url) = base_url.join(href) {
                return Cow::Owned(new_url.into());
            }
        };
        // If it is not a valid URL and there is no base URL specified, we assume a local path
        Cow::Borrowed(href)
    }
}

fn load_external(location: &str) -> Result<String> {
    if location.starts_with("https") | location.starts_with("http") {
        #[cfg(feature = "http")]
        {
            let request = attohttpc::RequestBuilder::try_new(attohttpc::Method::GET, location)?;
            let response = request.send()?;
            Ok(response.text()?)
        }

        #[cfg(not(feature = "http"))]
        {
            Err(InlineError::IO(std::io::Error::new(
                ErrorKind::Unsupported,
                "Loading external URLs requires the `http` feature",
            )))
        }
    } else {
        #[cfg(feature = "file")]
        {
            let location = location.trim_start_matches("file://");
            std::fs::read_to_string(location).map_err(|error| match error.kind() {
                ErrorKind::NotFound => InlineError::MissingStyleSheet {
                    path: location.to_string(),
                },
                _ => InlineError::IO(error),
            })
        }
        #[cfg(not(feature = "file"))]
        {
            Err(InlineError::IO(std::io::Error::new(
                ErrorKind::Unsupported,
                "Loading local files requires the `file` feature",
            )))
        }
    }
}

impl Default for CSSInliner<'_> {
    #[inline]
    fn default() -> Self {
        CSSInliner::new(InlineOptions::default())
    }
}

/// Shortcut for inlining CSS with default parameters.
///
/// # Errors
///
/// Inlining might fail for the following reasons:
///   - Missing stylesheet file;
///   - Remote stylesheet is not available;
///   - IO errors;
///   - Internal CSS selector parsing error;
#[inline]
pub fn inline(html: &str) -> Result<String> {
    CSSInliner::default().inline(html)
}

/// Shortcut for inlining CSS with default parameters and writing the output to a generic writer.
/// # Errors
///
/// Inlining might fail for the following reasons:
///   - Missing stylesheet file;
///   - Remote stylesheet is not available;
///   - IO errors;
///   - Internal CSS selector parsing error;
#[inline]
pub fn inline_to<W: Write>(html: &str, target: &mut W) -> Result<()> {
    CSSInliner::default().inline_to(html, target)
}
