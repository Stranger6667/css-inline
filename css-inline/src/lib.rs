//! # css-inline
//!
//! A crate for inlining CSS into HTML documents. When you send HTML emails, you need to use "style"
//! attributes instead of "style" tags.
//!
//! For example, this HTML:
//!
//! ```html
//! <html>
//!     <head>
//!         <title>Test</title>
//!         <style>h1 { color:blue; }</style>
//!     </head>
//!     <body>
//!         <h1>Big Text</h1>
//!     </body>
//! </html>
//! ```
//!
//! Will be turned into this:
//!
//! ```html
//! <html>
//!     <head><title>Test</title></head>
//!     <body>
//!         <h1 style="color:blue;">Big Text</h1>
//!     </body>
//! </html>
//! ```
//!
//! ## Usage
//!
//! ```rust
//! const HTML: &str = r#"<html>
//! <head>
//!     <title>Test</title>
//!     <style>h1 { color:blue; }</style>
//! </head>
//! <body>
//!     <h1>Big Text</h1>
//! </body>
//! </html>"#;
//!
//!fn main() -> Result<(), css_inline::InlineError> {
//!    let inlined = css_inline::inline(HTML)?;  // shortcut with default options
//!    // Do something with inlined HTML, e.g. send an email
//!    Ok(())
//! }
//! ```
//!
//! ### Features & Configuration
//!
//! `css-inline` can be configured by using `CSSInliner::options()` that implements the Builder pattern:
//!
//! ```rust
//! const HTML: &str = r#"<html>
//! <head>
//!     <title>Test</title>
//!     <style>h1 { color:blue; }</style>
//! </head>
//! <body>
//!     <h1>Big Text</h1>
//! </body>
//! </html>"#;
//!
//! fn main() -> Result<(), css_inline::InlineError> {
//!     let inliner = css_inline::CSSInliner::options()
//!         .load_remote_stylesheets(false)
//!         .build();
//!     let inlined = inliner.inline(HTML);
//!     // Do something with inlined HTML, e.g. send an email
//!     Ok(())
//! }
//! ```
//!
//! - `inline_style_tags`. Whether to inline CSS from "style" tags. Default: `true`
//! - `remove_style_tags`. Remove "style" tags after inlining. Default: `false`
//! - `base_url`. Base URL to resolve relative URLs. Default: `None`
//! - `load_remote_stylesheets`. Whether remote stylesheets should be loaded or not. Default: `true`
//! - `extra_css`. Additional CSS to inline. Default: `None`
//!
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

use ahash::AHashMap;
pub use error::InlineError;
use smallvec::{smallvec, SmallVec};
use std::{borrow::Cow, collections::hash_map::Entry, fs, io::Write};
pub use url::{ParseError, Url};

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
    #[inline]
    pub fn inline(&self, html: &str) -> Result<String> {
        // Allocating the same amount of memory as the input HTML helps to avoid
        // some allocations, but most probably the output size will be different than
        // the original HTML
        let mut out = Vec::with_capacity(html.len());
        self.inline_to(html, &mut out)?;
        Ok(String::from_utf8_lossy(&out).to_string())
    }

    /// Inline CSS & write the result to a generic writer. Use it if you want to write
    /// the inlined document to a file.
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
        let mut styles = AHashMap::with_capacity(128);
        let mut style_tags: SmallVec<[NodeDataRef<ElementData>; 4]> = smallvec![];
        if self.options.inline_style_tags {
            for style_tag in document
                .select("style")
                .map_err(|_| error::InlineError::ParseError(Cow::from("Unknown error")))?
            {
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
                .filter_map(|link_tag| link_tag.attributes.borrow().get("href").map(str::to_string))
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
                if let Some(existing_style) = attributes.get_mut("style") {
                    *existing_style = merge_styles(existing_style, &styles)?;
                } else {
                    let mut final_styles = String::with_capacity(128);
                    for (name, (_, value)) in styles {
                        final_styles.push_str(name.as_str());
                        final_styles.push(':');
                        final_styles.push_str(value.as_str());
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

fn load_external(url: &str) -> Result<String> {
    if url.starts_with("https") | url.starts_with("http") {
        let response = attohttpc::get(url).send()?;
        Ok(response.text()?)
    } else {
        fs::read_to_string(url).map_err(InlineError::IO)
    }
}

type NodeId = *const Node;

#[allow(clippy::mutable_key_type)]
fn process_css(
    document: &NodeRef,
    css: &str,
    styles: &mut AHashMap<NodeId, AHashMap<String, (Specificity, String)>>,
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
                        .or_insert_with(|| AHashMap::with_capacity(8));
                    for (name, value) in &declarations {
                        match element_styles.entry(name.to_string()) {
                            Entry::Occupied(mut entry) => {
                                if entry.get().0 <= specificity {
                                    entry.insert((specificity, (*value).to_string()));
                                }
                            }
                            Entry::Vacant(entry) => {
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
#[inline]
pub fn inline(html: &str) -> Result<String> {
    CSSInliner::default().inline(html)
}

/// Shortcut for inlining CSS with default parameters and writing the output to a generic writer.
#[inline]
pub fn inline_to<W: Write>(html: &str, target: &mut W) -> Result<()> {
    CSSInliner::default().inline_to(html, target)
}

fn merge_styles(
    existing_style: &str,
    new_styles: &AHashMap<String, (Specificity, String)>,
) -> Result<String> {
    // Parse existing declarations in the "style" attribute
    let mut input = cssparser::ParserInput::new(existing_style);
    let mut parser = cssparser::Parser::new(&mut input);
    let declarations =
        cssparser::DeclarationListParser::new(&mut parser, parser::CSSDeclarationListParser);
    // New rules should not override old ones and we store selectors inline to check the old rules later
    let mut buffer: SmallVec<[String; 8]> = smallvec![];
    let mut final_styles = String::with_capacity(256);
    for declaration in declarations {
        let (name, value) = declaration?;
        final_styles.push_str(&name);
        final_styles.push(':');
        final_styles.push_str(value);
        final_styles.push(';');
        // This property won't be taken from new styles
        buffer.push(name.to_string())
    }
    for (property, (_, value)) in new_styles {
        if !buffer.contains(property) {
            final_styles.push_str(property);
            final_styles.push(':');
            final_styles.push_str(value);
            final_styles.push(';');
        }
    }
    Ok(final_styles)
}
