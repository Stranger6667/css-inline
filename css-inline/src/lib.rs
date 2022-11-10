#![doc = include_str!("../README.md")]
#![warn(
    clippy::doc_markdown,
    clippy::redundant_closure,
    clippy::explicit_iter_loop,
    clippy::match_same_arms,
    clippy::needless_borrow,
    clippy::print_stdout,
    clippy::integer_arithmetic,
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
    rust_2018_compatibility
)]

use kuchiki::{
    parse_html, traits::TendrilSink, ElementData, Node, NodeDataRef, NodeRef, Specificity,
};

pub mod error;
mod parser;

pub use error::InlineError;
use indexmap::IndexMap;
use smallvec::{smallvec, SmallVec};
use std::{
    borrow::Cow,
    io::{ErrorKind, Write},
};

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
    /// Whether to inline CSS from "style" tags.
    pub inline_style_tags: bool,
    /// Remove "style" tags after inlining.
    pub remove_style_tags: bool,
    /// Used for loading external stylesheets via relative URLs.
    pub base_url: Option<Url>,
    /// Whether remote stylesheets should be loaded or not.
    pub load_remote_stylesheets: bool,
    // The point of using `Cow` here is Python bindings, where it is problematic to pass a reference
    // without dealing with memory leaks & unsafe. With `Cow` we can use moved values as `String` in
    // Python wrapper for `CSSInliner` and `&str` in Rust & simple functions on the Python side
    /// Additional CSS to inline.
    pub extra_css: Option<Cow<'a, str>>,
}

impl<'a> InlineOptions<'a> {
    /// Options for "compact" HTML output.
    #[must_use]
    #[inline]
    pub const fn compact() -> Self {
        InlineOptions {
            inline_style_tags: true,
            remove_style_tags: true,
            base_url: None,
            load_remote_stylesheets: true,
            extra_css: None,
        }
    }

    /// Override whether "style" tags should be inlined.
    #[must_use]
    pub fn inline_style_tags(mut self, inline_style_tags: bool) -> Self {
        self.inline_style_tags = inline_style_tags;
        self
    }

    /// Override whether "style" tags should be removed after processing.
    #[must_use]
    pub fn remove_style_tags(mut self, remove_style_tags: bool) -> Self {
        self.remove_style_tags = remove_style_tags;
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
            remove_style_tags: false,
            base_url: None,
            load_remote_stylesheets: true,
            extra_css: None,
        }
    }
}

type Result<T> = std::result::Result<T, InlineError>;
const CSS_INLINE_ATTRIBUTE: &str = "data-css-inline";

/// Customizable CSS inliner.
#[derive(Debug)]
pub struct CSSInliner<'a> {
    options: InlineOptions<'a>,
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
    /// use url::Url;
    /// use css_inline::CSSInliner;
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

    /// Inliner, that will produce "compact" HTML.
    /// For example, "style" tags will be removed.
    #[must_use]
    #[inline]
    pub const fn compact() -> Self {
        CSSInliner {
            options: InlineOptions::compact(),
        }
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
        let mut out = Vec::with_capacity(html.len().saturating_mul(2));
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
        let document = parse_html().one(html);
        // CSS rules may overlap, and the final set of rules applied to an element depend on
        // selectors' specificity - selectors with higher specificity have more priority.
        // Inlining happens in two major steps:
        //   1. All available styles are mapped to respective elements together with their
        //      selector's specificity. When two rules overlap on the same declaration, then
        //      the one with higher specificity replaces another.
        //   2. Resulting styles are merged into existing "style" tags.
        #[allow(clippy::mutable_key_type)]
        // Each matched element is identified by their raw pointers - they are evaluated once
        // and then reused, which allows O(1) access to find them.
        // Internally, their raw pointers are used to implement `Eq`, which seems like the only
        // reasonable approach to compare them (performance-wise).
        let mut styles = IndexMap::with_capacity(128);
        let mut style_tags: SmallVec<[NodeDataRef<ElementData>; 4]> = smallvec![];
        if self.options.inline_style_tags {
            for style_tag in document
                .select("style")
                .map_err(|_| InlineError::ParseError(Cow::from("Unknown error")))?
            {
                if style_tag.attributes.borrow().get(CSS_INLINE_ATTRIBUTE) == Some("ignore") {
                    continue;
                }
                if let Some(first_child) = style_tag.as_node().first_child() {
                    if let Some(css_cell) = first_child.as_text() {
                        process_css(&document, css_cell.borrow().as_str(), &mut styles);
                    }
                }
                if self.options.remove_style_tags {
                    style_tags.push(style_tag);
                }
            }
        }
        if self.options.remove_style_tags {
            if !self.options.inline_style_tags {
                style_tags.extend(
                    document
                        .select("style")
                        .map_err(|_| error::InlineError::ParseError(Cow::from("Unknown error")))?,
                );
            }
            for style_tag in &style_tags {
                style_tag.as_node().detach();
            }
        }

        if self.options.load_remote_stylesheets {
            let mut links = document
                .select("link[rel~=stylesheet]")
                .map_err(|_| error::InlineError::ParseError(Cow::from("Unknown error")))?
                .filter_map(|link_tag| {
                    if link_tag.attributes.borrow().get(CSS_INLINE_ATTRIBUTE) == Some("ignore") {
                        None
                    } else {
                        link_tag.attributes.borrow().get("href").map(str::to_string)
                    }
                })
                .filter(|link| !link.is_empty())
                .collect::<Vec<String>>();
            links.sort_unstable();
            links.dedup();
            for href in &links {
                let url = self.get_full_url(href);
                let css = load_external(url.as_ref())?;
                process_css(&document, css.as_str(), &mut styles);
            }
        }
        if let Some(extra_css) = &self.options.extra_css {
            process_css(&document, extra_css, &mut styles);
        }
        for (node_id, styles) in styles {
            // SAFETY: All nodes are alive as long as `document` is in scope.
            // Therefore, any `document` children should be alive and it is safe to dereference
            // pointers to them
            let node = unsafe { &*node_id };
            // It can be borrowed if the current selector matches <link> tag, that is
            // already borrowed in `inline_to`. We can ignore such matches
            if let Ok(mut attributes) = node
                .as_element()
                .expect("Element is expected")
                .attributes
                .try_borrow_mut()
            {
                // Skip inlining for tags that have `data-css-inline="ignore"` attribute
                if attributes.get(CSS_INLINE_ATTRIBUTE) == Some("ignore") {
                    continue;
                }
                if let Some(existing_style) = attributes.get_mut("style") {
                    *existing_style = merge_styles(existing_style, &styles)?;
                } else {
                    let mut final_styles = String::with_capacity(128);
                    let mut styles = styles.iter().collect::<Vec<_>>();
                    styles.sort_unstable_by(|(_, (a, _)), (_, (b, _))| a.cmp(b));
                    for (name, (_, value)) in styles {
                        final_styles.push_str(name.as_str());
                        final_styles.push(':');
                        replace_double_quotes!(final_styles, name, value);
                        final_styles.push(';');
                    }
                    attributes.insert("style", final_styles);
                };
            }
        }
        document.serialize(target)?;
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

fn load_external(mut location: &str) -> Result<String> {
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
            location = location.trim_start_matches("file://");
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

type NodeId = *const Node;

#[allow(clippy::mutable_key_type)]
fn process_css(
    document: &NodeRef,
    css: &str,
    styles: &mut IndexMap<NodeId, IndexMap<String, (Specificity, String)>>,
) {
    let mut parse_input = cssparser::ParserInput::new(css);
    let mut parser = cssparser::Parser::new(&mut parse_input);
    let rule_list =
        cssparser::RuleListParser::new_for_stylesheet(&mut parser, parser::CSSRuleListParser);
    for (selectors, declarations) in rule_list.flatten() {
        // Only CSS Syntax Level 3 is supported, therefore it is OK to split by `,`
        // With `is` or `where` selectors (Level 4) this split should be done on the parser level
        for selector in selectors.split(',') {
            if let Ok(matching_elements) = document.select(selector) {
                // There is always only one selector applied
                let specificity = matching_elements.selectors.0[0].specificity();
                for matching_element in matching_elements {
                    let element_styles = styles
                        .entry(&**matching_element.as_node())
                        .or_insert_with(|| IndexMap::with_capacity(8));
                    // Iterate over pairs of property name & value
                    // Example: `padding`, `0`
                    for (name, value) in &declarations {
                        match element_styles.entry(name.to_string()) {
                            indexmap::map::Entry::Occupied(mut entry) => {
                                if entry.get().0 <= specificity {
                                    entry.insert((specificity, (*value).to_string()));
                                }
                            }
                            indexmap::map::Entry::Vacant(entry) => {
                                entry.insert((specificity, (*value).to_string()));
                            }
                        }
                    }
                }
            }
            // Skip selectors that can't be parsed
            // Ignore not parsable entries. E.g. there is no parser for @media queries
            // Which means that they will fall into this category and will be ignored
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

fn merge_styles(
    existing_style: &str,
    new_styles: &IndexMap<String, (Specificity, String)>,
) -> Result<String> {
    // Parse existing declarations in the "style" attribute
    let mut input = cssparser::ParserInput::new(existing_style);
    let mut parser = cssparser::Parser::new(&mut input);
    let declarations =
        cssparser::DeclarationListParser::new(&mut parser, parser::CSSDeclarationListParser);
    // New rules should not override old ones unless !important and we store selectors inline to check the old rules later
    let mut buffer: SmallVec<[String; 8]> = smallvec![];
    let mut final_styles: Vec<String> = Vec::new();
    for declaration in declarations {
        let (name, value) = declaration?;
        let mut style = String::with_capacity(256);
        style.push_str(&name);
        style.push_str(": ");
        replace_double_quotes!(style, name, value.trim());
        final_styles.push(style);
        // This property won't be taken from new styles unless it's !important
        buffer.push(name.to_string());
    }
    let mut new_styles = new_styles.iter().collect::<Vec<_>>();
    new_styles.sort_unstable_by(|(_, (a, _)), (_, (b, _))| a.cmp(b));
    for (property, (_, value)) in new_styles {
        match (
            value.strip_suffix("!important"),
            buffer.iter().position(|r| r == property),
        ) {
            // The new rule is `!important` and there is already one in existing styles:
            // override with the new one.
            #[allow(clippy::integer_arithmetic)]
            (Some(value), Some(index)) => {
                // Reuse existing allocation
                let target = &mut final_styles[index];
                // Keep '<rule>: ` (with a space at the end)
                // NOTE: There will be no overflow as the new len is always smaller than the old one
                target.truncate(property.len() + 2);
                // And push the value
                target.push_str(value.trim());
            }
            // No such rules exist - push the version with `!important` trimmed
            (Some(value), None) => final_styles.push(format!("{}: {}", property, value.trim())),
            // Completely new rule - write it
            (None, None) => final_styles.push(format!("{}: {}", property, value.trim())),
            // Rule exists and the new one is not `!important` - keep the original one
            (None, Some(_)) => {}
        }
    }
    Ok(final_styles.join(";"))
}
