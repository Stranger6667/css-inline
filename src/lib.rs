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
//!         <style>
//!             h1, h2 { color:blue; }
//!             strong { text-decoration:none }
//!             p { font-size:2px }
//!             p.footer { font-size: 1px}
//!         </style>
//!     </head>
//!     <body>
//!         <h1>Big Text</h1>
//!         <p>
//!             <strong>Solid</strong>
//!         </p>
//!         <p class="footer">Foot notes</p>
//!     </body>
//! </html>
//! ```
//!
//! Will be turned into this:
//!
//! ```html
//! <html>
//!     <head>
//!         <title>Test</title>
//!     </head>
//!     <body>
//!         <h1 style="color:blue;">Big Text</h1>
//!         <p style="font-size:2px;">
//!             <strong style="text-decoration:none;">Solid</strong>
//!         </p>
//!         <p style="font-size:1px;">Foot notes</p>
//!     </body>
//! </html>
//! ```
//!
//! ## Example:
//!
//! ```rust
//! const HTML: &str = r#"<html>
//! <head>
//!     <title>Test</title>
//!     <style>
//!         h1, h2 { color:blue; }
//!         strong { text-decoration:none }
//!         p { font-size:2px }
//!         p.footer { font-size: 1px}
//!     </style>
//! </head>
//! <body>
//!     <h1>Big Text</h1>
//!     <p>
//!         <strong>Solid</strong>
//!     </p>
//!     <p class="footer">Foot notes</p>
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
//! Or if you need more control over inlining you could use `CSSInliner`:
//!
//! ```rust
//! const HTML: &str = r#"<html>
//! <head>
//!     <title>Test</title>
//!     <style>
//!         h1, h2 { color:blue; }
//!         strong { text-decoration:none }
//!         p { font-size:2px }
//!         p.footer { font-size: 1px}
//!     </style>
//! </head>
//! <body>
//!     <h1>Big Text</h1>
//!     <p>
//!         <strong>Solid</strong>
//!     </p>
//!     <p class="footer">Foot notes</p>
//! </body>
//! </html>"#;
//!
//!fn main() -> Result<(), css_inline::InlineError> {
//!    let options = css_inline::InlineOptions { remove_style_tags: true };
//!    let inliner = css_inline::CSSInliner::new(options);
//!    let inlined = inliner.inline(HTML)?;
//!    // Do something with inlined HTML, e.g. send an email
//!    Ok(())
//! }
//!```
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
use crate::parse::Declaration;
use kuchiki::traits::TendrilSink;
use kuchiki::{parse_html, Selectors};

pub mod error;
mod parse;

pub use error::InlineError;
use std::collections::HashMap;

#[derive(Debug)]
struct Rule {
    selectors: Selectors,
    declarations: Vec<Declaration>,
}

impl Rule {
    pub fn new(selectors: &str, declarations: Vec<Declaration>) -> Result<Rule, ()> {
        Ok(Rule {
            selectors: Selectors::compile(selectors)?,
            declarations,
        })
    }
}

/// Configuration options for CSS inlining process.
#[derive(Debug)]
pub struct InlineOptions {
    /// Remove "style" tags after inlining
    pub remove_style_tags: bool,
}

impl Default for InlineOptions {
    #[inline]
    fn default() -> Self {
        InlineOptions {
            remove_style_tags: false,
        }
    }
}

/// Customizable CSS inliner.
#[derive(Debug)]
pub struct CSSInliner {
    options: InlineOptions,
}

impl CSSInliner {
    /// Create a new `CSSInliner` instance with given options.
    #[inline]
    pub fn new(options: InlineOptions) -> Self {
        CSSInliner { options }
    }

    /// Inline CSS styles from <style> tags to matching elements in the HTML tree.
    #[inline]
    pub fn inline(&self, html: &str) -> Result<String, InlineError> {
        let document = parse_html().one(html);
        for style_tag in document
            .select("style")
            .map_err(|_| error::InlineError::ParseError("Unknown error".to_string()))?
        {
            if let Some(first_child) = style_tag.as_node().first_child() {
                if let Some(css_cell) = first_child.as_text() {
                    let css = css_cell.borrow();
                    let mut parse_input = cssparser::ParserInput::new(css.as_str());
                    let mut parser = parse::CSSParser::new(&mut parse_input);
                    for parsed in parser.parse() {
                        if let Ok((selector, declarations)) = parsed {
                            let rule = Rule::new(&selector, declarations).map_err(|_| {
                                error::InlineError::ParseError("Unknown error".to_string())
                            })?;
                            let matching_elements = document
                                .inclusive_descendants()
                                .filter_map(|node| node.into_element_ref())
                                .filter(|element| rule.selectors.matches(element));
                            for matching_element in matching_elements {
                                let mut attributes = matching_element.attributes.borrow_mut();
                                let style = if let Some(existing_style) = attributes.get("style") {
                                    merge_styles(existing_style, &rule.declarations)?
                                } else {
                                    rule.declarations
                                        .iter()
                                        .map(|&(ref key, ref value)| format!("{}:{};", key, value))
                                        .collect()
                                };
                                attributes.insert("style", style);
                            }
                        }
                        // Ignore not parsable entries. E.g. there is no parser for @media queries
                        // Which means that they will fall into this category and will be ignored
                    }
                }
            }
            if self.options.remove_style_tags {
                style_tag.as_node().detach()
            }
        }
        let mut out = vec![];
        document.serialize(&mut out)?;
        Ok(String::from_utf8_lossy(&out).to_string())
    }
}

impl Default for CSSInliner {
    #[inline]
    fn default() -> Self {
        CSSInliner::new(Default::default())
    }
}

/// Shortcut for inlining CSS with default parameters.
#[inline]
pub fn inline(html: &str) -> Result<String, InlineError> {
    CSSInliner::default().inline(html)
}

fn merge_styles(existing_style: &str, new_styles: &[Declaration]) -> Result<String, InlineError> {
    // Parse existing declarations in "style" attribute
    let mut input = cssparser::ParserInput::new(existing_style);
    let mut parser = cssparser::Parser::new(&mut input);
    let declarations =
        cssparser::DeclarationListParser::new(&mut parser, parse::CSSDeclarationListParser);
    // Merge existing with the new ones
    // We know that at least one rule already exists, so we add 1
    let mut styles: HashMap<String, String> =
        HashMap::with_capacity(new_styles.len().saturating_add(1));
    for declaration in declarations.into_iter() {
        let (property, value) = declaration?;
        styles.insert(property, value);
    }
    for (property, value) in new_styles.iter() {
        styles.insert(property.to_string(), value.to_string());
    }
    // Create a new declarations list
    Ok(styles
        .iter()
        .map(|(key, value)| format!("{}:{};", key, value))
        .collect::<String>())
}
