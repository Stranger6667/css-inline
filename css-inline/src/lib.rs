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
#![allow(clippy::module_name_repetitions)]
pub mod error;
mod hasher;
mod html;
mod parser;
mod resolver;

pub use error::InlineError;
use indexmap::IndexMap;
#[cfg(feature = "stylesheet-cache")]
use lru::{DefaultHasher, LruCache};
use std::{borrow::Cow, fmt::Formatter, hash::BuildHasherDefault, io::Write, sync::Arc};

use crate::html::ElementStyleMap;
use hasher::BuildNoHashHasher;
use html::{Document, InliningMode};
pub use resolver::{DefaultStylesheetResolver, StylesheetResolver};
pub use url::{ParseError, Url};

/// An LRU Cache for external stylesheets.
#[cfg(feature = "stylesheet-cache")]
pub type StylesheetCache<S = DefaultHasher> = LruCache<String, String, S>;

/// Configuration options for CSS inlining process.
#[allow(clippy::struct_excessive_bools)]
pub struct InlineOptions<'a> {
    /// Whether to inline CSS from "style" tags.
    ///
    /// Sometimes HTML may include a lot of boilerplate styles, that are not applicable in every
    /// scenario and it is useful to ignore them and use `extra_css` instead.
    pub inline_style_tags: bool,
    /// Keep "style" tags after inlining.
    pub keep_style_tags: bool,
    /// Keep "link" tags after inlining.
    pub keep_link_tags: bool,
    /// Used for loading external stylesheets via relative URLs.
    pub base_url: Option<Url>,
    /// Whether remote stylesheets should be loaded or not.
    pub load_remote_stylesheets: bool,
    /// External stylesheet cache.
    #[cfg(feature = "stylesheet-cache")]
    pub cache: Option<std::sync::Mutex<StylesheetCache>>,
    // The point of using `Cow` here is Python bindings, where it is problematic to pass a reference
    // without dealing with memory leaks & unsafe. With `Cow` we can use moved values as `String` in
    // Python wrapper for `CSSInliner` and `&str` in Rust & simple functions on the Python side
    /// Additional CSS to inline.
    pub extra_css: Option<Cow<'a, str>>,
    /// Pre-allocate capacity for HTML nodes during parsing.
    /// It can improve performance when you have an estimate of the number of nodes in your HTML document.
    pub preallocate_node_capacity: usize,
    /// A way to resolve stylesheets from various sources.
    pub resolver: Arc<dyn StylesheetResolver>,
}

impl std::fmt::Debug for InlineOptions<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("InlineOptions");
        debug
            .field("inline_style_tags", &self.inline_style_tags)
            .field("keep_style_tags", &self.keep_style_tags)
            .field("keep_link_tags", &self.keep_link_tags)
            .field("base_url", &self.base_url)
            .field("load_remote_stylesheets", &self.load_remote_stylesheets);
        #[cfg(feature = "stylesheet-cache")]
        {
            debug.field("cache", &self.cache);
        }
        debug
            .field("extra_css", &self.extra_css)
            .field("preallocate_node_capacity", &self.preallocate_node_capacity)
            .finish_non_exhaustive()
    }
}

impl<'a> InlineOptions<'a> {
    /// Override whether "style" tags should be inlined.
    #[must_use]
    pub fn inline_style_tags(mut self, inline_style_tags: bool) -> Self {
        self.inline_style_tags = inline_style_tags;
        self
    }

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

    /// Set external stylesheet cache.
    #[must_use]
    #[cfg(feature = "stylesheet-cache")]
    pub fn cache(mut self, cache: impl Into<Option<StylesheetCache>>) -> Self {
        if let Some(cache) = cache.into() {
            self.cache = Some(std::sync::Mutex::new(cache));
        } else {
            self.cache = None;
        }
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

    /// Set the way to resolve stylesheets from various sources.
    #[must_use]
    pub fn resolver(mut self, resolver: Arc<dyn StylesheetResolver>) -> Self {
        self.resolver = resolver;
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
            inline_style_tags: true,
            keep_style_tags: false,
            keep_link_tags: false,
            base_url: None,
            load_remote_stylesheets: true,
            #[cfg(feature = "stylesheet-cache")]
            cache: None,
            extra_css: None,
            preallocate_node_capacity: 32,
            resolver: Arc::new(DefaultStylesheetResolver),
        }
    }
}

/// A specialized `Result` type for CSS inlining operations.
pub type Result<T> = std::result::Result<T, InlineError>;

/// Customizable CSS inliner.
#[derive(Debug)]
pub struct CSSInliner<'a> {
    options: InlineOptions<'a>,
}

const GROWTH_COEFFICIENT: f64 = 1.5;
// A rough coefficient to calculate the number of individual declarations based on the total CSS size.
const DECLARATION_SIZE_COEFFICIENT: f64 = 30.0;

fn allocate_output_buffer(html: &str) -> Vec<u8> {
    // Allocating more memory than the input HTML, as the inlined version is usually bigger
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    Vec::with_capacity(
        (html.len() as f64 * GROWTH_COEFFICIENT)
            .min(usize::MAX as f64)
            .round() as usize,
    )
}

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
    ///
    /// # Panics
    ///
    /// This function may panic if external stylesheet cache lock is poisoned, i.e. another thread
    /// using the same inliner panicked while resolving external stylesheets.
    #[inline]
    pub fn inline(&self, html: &str) -> Result<String> {
        let mut out = allocate_output_buffer(html);
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
    ///
    /// # Panics
    ///
    /// This function may panic if external stylesheet cache lock is poisoned, i.e. another thread
    /// using the same inliner panicked while resolving external stylesheets.
    #[inline]
    pub fn inline_to<W: Write>(&self, html: &str, target: &mut W) -> Result<()> {
        self.inline_to_impl(html, None, target, InliningMode::Document)
    }

    /// Inline CSS into an HTML fragment.
    ///
    /// # Errors
    ///
    /// Inlining might fail for the following reasons:
    ///   - Missing stylesheet file;
    ///   - Remote stylesheet is not available;
    ///   - IO errors;
    ///   - Internal CSS selector parsing error;
    ///
    /// # Panics
    ///
    /// This function may panic if external stylesheet cache lock is poisoned, i.e. another thread
    /// using the same inliner panicked while resolving external stylesheets.
    pub fn inline_fragment(&self, html: &str, css: &str) -> Result<String> {
        let mut out = allocate_output_buffer(html);
        self.inline_fragment_to(html, css, &mut out)?;
        Ok(String::from_utf8_lossy(&out).to_string())
    }

    /// Inline CSS into an HTML fragment and write the result to a generic writer.
    ///
    /// # Errors
    ///
    /// Inlining might fail for the following reasons:
    ///   - Missing stylesheet file;
    ///   - Remote stylesheet is not available;
    ///   - IO errors;
    ///   - Internal CSS selector parsing error;
    ///
    /// # Panics
    ///
    /// This function may panic if external stylesheet cache lock is poisoned, i.e. another thread
    /// using the same inliner panicked while resolving external stylesheets.
    pub fn inline_fragment_to<W: Write>(
        &self,
        html: &str,
        css: &str,
        target: &mut W,
    ) -> Result<()> {
        self.inline_to_impl(html, Some(css), target, InliningMode::Fragment)
    }

    #[allow(clippy::too_many_lines)]
    fn inline_to_impl<W: Write>(
        &self,
        html: &str,
        css: Option<&str>,
        target: &mut W,
        mode: InliningMode,
    ) -> Result<()> {
        let document = Document::parse_with_options(
            html.as_bytes(),
            self.options.preallocate_node_capacity,
            mode,
        );
        // CSS rules may overlap, and the final set of rules applied to an element depend on
        // selectors' specificity - selectors with higher specificity have more priority.
        // Inlining happens in two major steps:
        //   1. All available styles are mapped to respective elements together with their
        //      selector's specificity. When two rules overlap on the same declaration, then
        //      the one with higher specificity replaces another.
        //   2. Resulting styles are merged into existing "style" tags.
        let mut size_estimate: usize = if self.options.inline_style_tags {
            document
                .styles()
                .map(|s| {
                    // Add 1 to account for the extra `\n` char we add between styles
                    s.len().saturating_add(1)
                })
                .sum()
        } else {
            0
        };
        if let Some(extra_css) = &self.options.extra_css {
            size_estimate = size_estimate.saturating_add(extra_css.len());
        }
        if let Some(css) = css {
            size_estimate = size_estimate.saturating_add(css.len());
        }
        let mut raw_styles = String::with_capacity(size_estimate);
        if self.options.inline_style_tags {
            for style in document.styles() {
                raw_styles.push_str(style);
                raw_styles.push('\n');
            }
        }
        if self.options.load_remote_stylesheets {
            let mut links = document.stylesheets().collect::<Vec<&str>>();
            links.sort_unstable();
            links.dedup();
            for href in &links {
                let url = self.get_full_url(href);
                #[cfg(feature = "stylesheet-cache")]
                if let Some(lock) = self.options.cache.as_ref() {
                    let mut cache = lock.lock().expect("Cache lock is poisoned");
                    if let Some(cached) = cache.get(url.as_ref()) {
                        raw_styles.push_str(cached);
                        raw_styles.push('\n');
                        continue;
                    }
                }

                let css = self.options.resolver.retrieve(url.as_ref())?;
                raw_styles.push_str(&css);
                raw_styles.push('\n');

                #[cfg(feature = "stylesheet-cache")]
                if let Some(lock) = self.options.cache.as_ref() {
                    let mut cache = lock.lock().expect("Cache lock is poisoned");
                    cache.put(url.into_owned(), css);
                }
            }
        }
        if let Some(extra_css) = &self.options.extra_css {
            raw_styles.push_str(extra_css);
        }
        if let Some(css) = css {
            raw_styles.push_str(css);
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
        for rule in cssparser::StyleSheetParser::new(
            &mut parser,
            &mut parser::CSSRuleListParser::new(&mut declarations),
        )
        .flatten()
        {
            rule_list.push(rule);
        }
        let mut caches = document.build_caches();
        for (selectors, (start, end)) in &rule_list {
            // Only CSS Syntax Level 3 is supported, therefore it is OK to split by `,`
            // With `is` or `where` selectors (Level 4) this split should be done on the parser level
            for selector in selectors.split(',') {
                if let Ok(matching_elements) = document.select(selector, &mut caches) {
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
                                    match (
                                        value.contains("!important"),
                                        entry.get().1.contains("!important"),
                                    ) {
                                        // Equal importance; the higher specificity wins.
                                        (false, false) | (true, true) => {
                                            if entry.get().0 <= specificity {
                                                entry.insert((specificity, *value));
                                            }
                                        }
                                        // Only the new value is important; it wins.
                                        (true, false) => {
                                            entry.insert((specificity, *value));
                                        }
                                        // The old value is important and the new one is not; keep
                                        // the old value.
                                        (false, true) => {}
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
            mode,
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
///
/// # Panics
///
/// This function may panic if external stylesheet cache lock is poisoned, i.e. another thread
/// using the same inliner panicked while resolving external stylesheets.
#[inline]
pub fn inline(html: &str) -> Result<String> {
    CSSInliner::default().inline(html)
}

/// Shortcut for inlining CSS with default parameters and writing the output to a generic writer.
///
/// # Errors
///
/// Inlining might fail for the following reasons:
///   - Missing stylesheet file;
///   - Remote stylesheet is not available;
///   - IO errors;
///   - Internal CSS selector parsing error;
///
/// # Panics
///
/// This function may panic if external stylesheet cache lock is poisoned, i.e. another thread
/// using the same inliner panicked while resolving external stylesheets.
#[inline]
pub fn inline_to<W: Write>(html: &str, target: &mut W) -> Result<()> {
    CSSInliner::default().inline_to(html, target)
}

/// Shortcut for inlining CSS into an HTML fragment with default parameters.
///
/// # Errors
///
/// Inlining might fail for the following reasons:
///   - Missing stylesheet file;
///   - Remote stylesheet is not available;
///   - IO errors;
///   - Internal CSS selector parsing error;
///
/// # Panics
///
/// This function may panic if external stylesheet cache lock is poisoned, i.e. another thread
/// using the same inliner panicked while resolving external stylesheets.
#[inline]
pub fn inline_fragment(html: &str, css: &str) -> Result<String> {
    CSSInliner::default().inline_fragment(html, css)
}

/// Shortcut for inlining CSS into an HTML fragment with default parameters and writing the output to a generic writer.
///
/// # Errors
///
/// Inlining might fail for the following reasons:
///   - Missing stylesheet file;
///   - Remote stylesheet is not available;
///   - IO errors;
///   - Internal CSS selector parsing error;
///
/// # Panics
///
/// This function may panic if external stylesheet cache lock is poisoned, i.e. another thread
/// using the same inliner panicked while resolving external stylesheets.
#[inline]
pub fn inline_fragment_to<W: Write>(html: &str, css: &str, target: &mut W) -> Result<()> {
    CSSInliner::default().inline_fragment_to(html, css, target)
}

#[cfg(test)]
mod tests {
    use crate::{CSSInliner, InlineOptions};

    #[test]
    fn test_inliner_sync_send() {
        fn assert_send<T: Send + Sync>() {}
        assert_send::<CSSInliner<'_>>();
        assert_send::<InlineOptions<'_>>();
    }
}
