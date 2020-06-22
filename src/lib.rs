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
//!    let inlined = css_inline::inline(HTML)?;
//!    // Do something with inlined HTML, e.g. send an email
//!    Ok(())
//! }
//!
//! ```
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
use kuchiki::{parse_html, ElementData, NodeDataRef, Selectors};

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

fn process_style_node(node: &NodeDataRef<ElementData>) -> Vec<Rule> {
    let css = node.text_contents();
    let mut parse_input = cssparser::ParserInput::new(css.as_str());
    let mut parser = parse::CSSParser::new(&mut parse_input);
    parser
        .parse()
        .filter_map(|r| {
            r.map(|(selector, declarations)| Rule::new(&selector, declarations))
                .ok()
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| error::InlineError::ParseError)
        .expect("Parsing error") // Should return Result instead
}

/// Inline CSS styles from <style> tags to matching elements in the HTML tree.
pub fn inline(html: &str) -> Result<String, InlineError> {
    let document = parse_html().one(html);
    let rules = document
        .select("style")
        .map_err(|_| error::InlineError::ParseError)?
        .map(|ref node| process_style_node(node))
        .flatten();

    for rule in rules {
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

    let mut out = vec![];
    document
        .select("html")
        .map_err(|_| error::InlineError::ParseError)?
        .next()
        .expect("HTML tag should be present") // Should it?
        .as_node()
        .serialize(&mut out)?;
    Ok(String::from_utf8_lossy(&out).to_string())
}

fn merge_styles(existing_style: &str, new_styles: &[Declaration]) -> Result<String, InlineError> {
    // Parse existing declarations in "style" attribute
    let mut input = cssparser::ParserInput::new(existing_style);
    let mut parser = cssparser::Parser::new(&mut input);
    let declarations =
        cssparser::DeclarationListParser::new(&mut parser, parse::CSSDeclarationListParser);
    // Merge existing with the new ones
    let mut styles: HashMap<String, String> = HashMap::new();
    for declaration in declarations.into_iter() {
        let (property, value) = declaration?;
        styles.insert(property.to_string(), value.to_string());
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
