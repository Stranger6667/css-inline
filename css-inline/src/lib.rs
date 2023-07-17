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
use html5ever::tendril::StrTendril;
use indexmap::IndexMap;
use smallvec::{smallvec, SmallVec};
use std::{
    borrow::Cow,
    collections::btree_map::Entry,
    io::{ErrorKind, Write},
};

use hasher::BuildNoHashHasher;
use html::{Document, Specificity};
pub use url::{ParseError, Url};

/// Replace double quotes in property values.
///
/// This implementation is deliberately simplistic and covers only `font-family`, but escaping
/// might be needed in other properties that accept strings.
macro_rules! replace_double_quotes {
    ($target:expr, $name:expr, $value:expr) => {
        // Avoid allocation if there is no double quote in the input string
        if $name.starts_with("font-family") && memchr::memchr(b'"', $value.as_bytes()).is_some() {
            $target.push_str(&$value.replace('"', "\'"))
        } else {
            $target.push_str($value)
        };
    };
}

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
        let document = self.inline_impl(html)?;
        document.serialize(
            target,
            self.options.keep_style_tags,
            self.options.keep_link_tags,
        )?;
        Ok(())
    }

    /// Non-generic inlining function.
    fn inline_impl(&self, html: &str) -> Result<Document> {
        let mut document =
            Document::parse_with_options(html.as_bytes(), self.options.preallocate_node_capacity);
        // CSS rules may overlap, and the final set of rules applied to an element depend on
        // selectors' specificity - selectors with higher specificity have more priority.
        // Inlining happens in two major steps:
        //   1. All available styles are mapped to respective elements together with their
        //      selector's specificity. When two rules overlap on the same declaration, then
        //      the one with higher specificity replaces another.
        //   2. Resulting styles are merged into existing "style" tags.
        let mut size_estimate: usize = document.styles().map(str::len).sum();
        if let Some(extra_css) = &self.options.extra_css {
            size_estimate = size_estimate.saturating_add(extra_css.len());
        }
        let mut raw_styles = String::with_capacity(size_estimate);
        let mut styles = IndexMap::with_capacity_and_hasher(128, BuildNoHashHasher::default());
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
        let mut parse_input = cssparser::ParserInput::new(&raw_styles);
        let mut parser = cssparser::Parser::new(&mut parse_input);
        let rule_list: Vec<_> =
            cssparser::RuleListParser::new_for_stylesheet(&mut parser, parser::CSSRuleListParser)
                .flatten()
                .collect();
        for (selectors, declarations) in &rule_list {
            // Only CSS Syntax Level 3 is supported, therefore it is OK to split by `,`
            // With `is` or `where` selectors (Level 4) this split should be done on the parser level
            for selector in selectors.split(',') {
                if let Ok(matching_elements) = document.select(selector) {
                    let specificity = matching_elements.specificity();
                    for matching_element in matching_elements {
                        let element_styles =
                            styles.entry(matching_element.node_id).or_insert_with(|| {
                                IndexMap::<&str, (Specificity, &str)>::with_capacity(
                                    declarations.len(),
                                )
                            });
                        // Iterate over pairs of property name & value
                        // Example: `padding`, `0`
                        for (name, value) in declarations {
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
        let mut style_buffer: SmallVec<[String; 8]> = smallvec![];
        for (node_id, styles) in styles.iter_mut() {
            let element = if let Some(element) = document[*node_id].as_not_ignored_element_mut() {
                element
            } else {
                continue;
            };
            styles.sort_unstable_by(|_, (a, _), _, (b, _)| a.cmp(b));
            match element.attributes.get_style_entry() {
                Entry::Vacant(entry) => {
                    let estimated_declaration_size: usize = styles
                        .iter()
                        .map(|(name, (_, value))| {
                            name.len()
                                .saturating_add(STYLE_SEPARATOR.len())
                                .saturating_add(value.len())
                                // Additional byte for the semicolon symbol
                                .saturating_add(1)
                        })
                        .sum();
                    let mut buffer = String::with_capacity(estimated_declaration_size);
                    for (property, (_, value)) in styles {
                        write_declaration(&mut buffer, property, value);
                        buffer.push(';');
                    }
                    entry.insert(buffer.into());
                }
                Entry::Occupied(mut entry) => {
                    merge_styles(entry.get_mut(), styles, &mut style_buffer)?;
                }
            }
        }
        Ok(document)
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

const STYLE_SEPARATOR: &str = ": ";

macro_rules! push_or_update {
    ($style_buffer:expr, $length:expr, $name: expr, $value:expr) => {
        if let Some(style) = $style_buffer.get_mut($length) {
            style.clear();
            style.push_str($name);
            style.push_str(STYLE_SEPARATOR);
            style.push_str($value.trim());
        } else {
            let value = $value.trim();
            let mut style = String::with_capacity(
                $name
                    .len()
                    .saturating_add(STYLE_SEPARATOR.len())
                    .saturating_add(value.len()),
            );
            style.push_str($name);
            style.push_str(STYLE_SEPARATOR);
            style.push_str(value);
            $style_buffer.push(style);
            $length = $length.saturating_add(1);
        }
    };
}

#[inline]
fn write_declaration(style: &mut String, name: &str, value: &str) {
    style.push_str(name);
    style.push_str(STYLE_SEPARATOR);
    replace_double_quotes!(style, name, value.trim());
}

/// Merge a new set of styles into an current one, considering the rules of CSS precedence.
///
/// The merge process maintains the order of specificity and respects the `!important` rule in CSS.
fn merge_styles(
    current_style: &mut StrTendril,
    new_styles: &IndexMap<&str, (Specificity, &str)>,
    declarations_buffer: &mut SmallVec<[String; 8]>,
) -> Result<()> {
    // This function is designed with a focus on reusing existing allocations where possible
    // We start by parsing the current declarations in the "style" attribute
    let mut parser_input = cssparser::ParserInput::new(current_style);
    let mut parser = cssparser::Parser::new(&mut parser_input);
    let current_declarations =
        cssparser::DeclarationListParser::new(&mut parser, parser::CSSDeclarationListParser);
    // We manually manage the length of our buffer. The buffer may contain slots used
    // in previous runs, and we want to access only the portion that we build in this iteration
    let mut parsed_declarations_count: usize = 0;
    for (idx, declaration) in current_declarations.enumerate() {
        parsed_declarations_count = parsed_declarations_count.saturating_add(1);
        let (property, value) = declaration?;
        let estimated_declaration_size = property
            .len()
            .saturating_add(STYLE_SEPARATOR.len())
            .saturating_add(value.len());
        // We store the existing style declarations in the buffer for later merging with new styles
        // If possible, we reuse existing slots in the buffer to avoid additional allocations
        if let Some(buffer) = declarations_buffer.get_mut(idx) {
            buffer.clear();
            buffer.reserve(estimated_declaration_size);
            write_declaration(buffer, &property, value);
        } else {
            let mut buffer = String::with_capacity(estimated_declaration_size);
            write_declaration(&mut buffer, &property, value);
            declarations_buffer.push(buffer);
        };
    }
    // Next, we iterate over the new styles and merge them into our existing set
    // New rules will not override old ones unless they are marked as `!important`
    for (property, (_, value)) in new_styles {
        match (
            value.strip_suffix("!important"),
            declarations_buffer.iter_mut().find(|style| {
                style.starts_with(property)
                    && style.get(property.len()..=property.len().saturating_add(1))
                        == Some(STYLE_SEPARATOR)
            }),
        ) {
            // The new rule is `!important` and there's an existing rule with the same name
            // In this case, we override the existing rule with the new one
            (Some(value), Some(buffer)) => {
                // We keep the rule name and the colon-space suffix - '<rule>: `
                buffer.truncate(property.len().saturating_add(STYLE_SEPARATOR.len()));
                buffer.push_str(value.trim());
            }
            // There's no existing rule with the same name, but the new rule is `!important`
            // In this case, we add the new rule with the `!important` suffix removed
            (Some(value), None) => {
                push_or_update!(
                    declarations_buffer,
                    parsed_declarations_count,
                    property,
                    value
                );
            }
            // There's no existing rule with the same name, and the new rule is not `!important`
            // In this case, we just add the new rule as-is
            (None, None) => push_or_update!(
                declarations_buffer,
                parsed_declarations_count,
                property,
                value
            ),
            // Rule exists and the new one is not `!important` - leave the existing rule as-is and
            // ignore the new one.
            (None, Some(_)) => {}
        }
    }

    // We can now dispose of the parser input, which allows us to clear the current style string
    drop(parser_input);
    // Now we prepare to write the merged styles back into current style
    current_style.clear();
    let size_estimate: usize = declarations_buffer[..parsed_declarations_count]
        .iter()
        // Additional byte for the semicolon symbol
        .map(|s| s.len().saturating_add(1))
        .sum();

    // Reserve enough space in current style for the merged style string
    current_style.reserve(u32::try_from(size_estimate).expect("Size overflow"));

    // Write the merged styles into the current style
    for declaration in &declarations_buffer[..parsed_declarations_count] {
        if !current_style.is_empty() {
            current_style.push_char(';');
        }
        current_style.push_slice(declaration);
    }
    Ok(())
}
