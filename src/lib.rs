//! # css-inline
//!
//! A crate for inlining CSS into HTML documents. When you send HTML emails you need to use "style"
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
//! `css-inline` can be configured by using `InlineOptions` and `CSSInliner`:
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
//!     let options = css_inline::InlineOptions {
//!         load_remote_stylesheets: false,
//!         ..Default::default()
//!     };
//!     let inliner = css_inline::CSSInliner::new(options);
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
    clippy::result_unwrap_used,
    clippy::result_map_unwrap_or_else,
    clippy::option_unwrap_used,
    clippy::option_map_unwrap_or_else,
    clippy::option_map_unwrap_or,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    missing_docs,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]
use kuchiki::traits::TendrilSink;
use kuchiki::{parse_html, NodeRef};

pub mod error;
mod parser;

use cssparser::CowRcStr;
pub use error::InlineError;
use parser::Rule;
use smallvec::{smallvec, SmallVec};
use std::borrow::Cow;
use std::fs::File;
use std::io::{Read, Write};
pub use url::{ParseError, Url};

/// Configuration options for CSS inlining process.
#[derive(Debug)]
pub struct InlineOptions<'a> {
    /// Whether to inline CSS from "style" tags
    pub inline_style_tags: bool,
    /// Remove "style" tags after inlining
    pub remove_style_tags: bool,
    /// Used for loading external stylesheets via relative URLs
    pub base_url: Option<Url>,
    /// Whether remote stylesheets should be loaded or not
    pub load_remote_stylesheets: bool,
    // The point of using `Cow` here is Python bindings, where it is problematic to pass a reference
    // without dealing with memory leaks & unsafe. With `Cow` we can use moved values as `String` in
    // Python wrapper for `CSSInliner` and `&str` in Rust & simple functions on the Python side
    /// Additional CSS to inline
    pub extra_css: Option<Cow<'a, str>>,
}

impl InlineOptions<'_> {
    /// Options for "compact" HTML output
    #[inline]
    pub fn compact() -> Self {
        InlineOptions {
            inline_style_tags: true,
            remove_style_tags: true,
            base_url: None,
            load_remote_stylesheets: true,
            extra_css: None,
        }
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
    #[inline]
    pub fn new(options: InlineOptions<'a>) -> Self {
        CSSInliner { options }
    }

    /// Inliner, that will produce "compact" HTML.
    /// For example, "style" tags will be removed.
    #[inline]
    pub fn compact() -> Self {
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
        if self.options.inline_style_tags {
            for style_tag in document
                .select("style")
                .map_err(|_| error::InlineError::ParseError("Unknown error".to_string()))?
            {
                if let Some(first_child) = style_tag.as_node().first_child() {
                    if let Some(css_cell) = first_child.as_text() {
                        process_css(&document, css_cell.borrow().as_str())?;
                    }
                }
                if self.options.remove_style_tags {
                    style_tag.as_node().detach()
                }
            }
        } else if self.options.remove_style_tags {
            for style_tag in document
                .select("style")
                .map_err(|_| error::InlineError::ParseError("Unknown error".to_string()))?
            {
                style_tag.as_node().detach()
            }
        }
        if self.options.load_remote_stylesheets {
            for link_tag in document
                .select("link[rel~=stylesheet]")
                .map_err(|_| error::InlineError::ParseError("Unknown error".to_string()))?
            {
                if let Some(href) = &link_tag.attributes.borrow().get("href") {
                    let url = self.get_full_url(href);
                    let css = self.load_external(url.as_ref())?;
                    process_css(&document, css.as_str())?;
                }
            }
        }
        if let Some(extra_css) = &self.options.extra_css {
            process_css(&document, extra_css)?;
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
            } else {
                // Not a URL, then it is a relative URL
                if let Ok(new_url) = base_url.join(href) {
                    return Cow::Owned(new_url.into_string());
                }
            }
        };
        // If it is not a valid URL and there is no base URL specified, we assume a local path
        Cow::Borrowed(href)
    }

    fn load_external(&self, url: &str) -> Result<String> {
        if url.starts_with("http") | url.starts_with("https") {
            let response = attohttpc::get(url).send()?;
            Ok(response.text()?)
        } else {
            let mut file = File::open(url)?;
            let mut css = String::new();
            file.read_to_string(&mut css)?;
            Ok(css)
        }
    }
}

fn process_css(document: &NodeRef, css: &str) -> Result<()> {
    let mut parse_input = cssparser::ParserInput::new(css);
    let mut parser = parser::CSSParser::new(&mut parse_input);
    for parsed in parser.parse() {
        if let Ok((selector, declarations)) = parsed {
            if let Ok(rule) = Rule::new(selector, declarations) {
                let matching_elements = document
                    .inclusive_descendants()
                    .filter_map(|node| node.into_element_ref())
                    .filter(|element| rule.selectors.matches(element));
                for matching_element in matching_elements {
                    // It can be borrowed if the current selector matches <link> tag, that is
                    // already borrowed in `inline_to`. We can ignore such matches
                    if let Ok(mut attributes) = matching_element.attributes.try_borrow_mut() {
                        if let Some(existing_style) = attributes.get_mut("style") {
                            *existing_style = merge_styles(existing_style, &rule.declarations)?
                        } else {
                            let mut final_styles = String::with_capacity(64);
                            for (name, value) in &rule.declarations {
                                final_styles.push_str(name);
                                final_styles.push(':');
                                final_styles.push_str(value);
                                final_styles.push(';');
                            }
                            attributes.insert("style", final_styles);
                        };
                    }
                }
            }
            // Skip selectors that can't be parsed
        }
        // Ignore not parsable entries. E.g. there is no parser for @media queries
        // Which means that they will fall into this category and will be ignored
    }
    Ok(())
}

impl Default for CSSInliner<'_> {
    #[inline]
    fn default() -> Self {
        CSSInliner::new(Default::default())
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

fn merge_styles(existing_style: &str, new_styles: &[parser::Declaration]) -> Result<String> {
    // Parse existing declarations in "style" attribute
    let mut input = cssparser::ParserInput::new(existing_style);
    let mut parser = cssparser::Parser::new(&mut input);
    let declarations =
        cssparser::DeclarationListParser::new(&mut parser, parser::CSSDeclarationListParser);
    // New rules override old ones and we store selectors inline to check the old rules later
    let mut buffer: SmallVec<[&CowRcStr; 8]> = smallvec![];
    let mut final_styles = String::with_capacity(256);
    for (property, value) in new_styles {
        final_styles.push_str(property);
        final_styles.push(':');
        final_styles.push_str(value);
        final_styles.push(';');
        // This property won't be taken from existing styles. i.e. it is overridden by new styles
        buffer.push(property);
    }
    for declaration in declarations {
        let (name, value) = declaration?;
        // Usually this buffer is small and it is faster than checking a {Hash,BTree}Map
        if !buffer.contains(&&name) {
            final_styles.push_str(&name);
            final_styles.push(':');
            final_styles.push_str(value);
            final_styles.push(';');
        }
    }
    Ok(final_styles)
}
