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
use selectors::context::SelectorCaches;
use smallvec::SmallVec;
use std::{
    borrow::Cow,
    fmt::Formatter,
    hash::BuildHasherDefault,
    io::Write,
    ops::Range,
    sync::Arc,
};

use crate::html::ElementStyleMap;
use hasher::BuildNoHashHasher;
use html::{Document, InliningMode, NodeData, NodeId};
use html5ever::tendril::StrTendril;
use rustc_hash::FxHashMap;
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
    /// Keep "at-rules" after inlining.
    pub keep_at_rules: bool,
    /// Remove trailing semicolons and spaces between properties and values.
    pub minify_css: bool,
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
    /// Remove selectors that were successfully inlined from inline `<style>` blocks.
    pub remove_inlined_selectors: bool,
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
            .field("remove_inlined_selectors", &self.remove_inlined_selectors)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct CssChunk {
    range: Range<usize>,
    origin: CssOrigin,
}

#[derive(Debug)]
enum CssOrigin {
    StyleNode(NodeId),
    LinkedStylesheet,
    ExtraCss,
    FragmentCss,
}

type SelectorList<'i> = SmallVec<[&'i str; 2]>;

#[derive(Debug, Default)]
struct SelectorUsage<'i> {
    selectors: Option<&'i str>,
    declarations: (usize, usize),
    rule_id: usize,
    chunk_index: usize,
    matched: bool,
}

#[derive(Debug, Default)]
struct RuleRemainder<'i> {
    selectors: SelectorList<'i>,
    declarations: (usize, usize),
}

#[derive(Debug, Default)]
struct SelectorCleanupState<'i> {
    chunks: Vec<CssChunk>,
    usages: Vec<SelectorUsage<'i>>,
}

impl<'i> SelectorCleanupState<'i> {
    fn new(enabled: bool) -> Option<Self> {
        if enabled {
            Some(Self::default())
        } else {
            None
        }
    }

    fn record_usage(&mut self, usage: SelectorUsage<'i>) {
        self.usages.push(usage);
    }

    fn chunk_index_for_slice(&self, slice: &str, source: &str) -> Option<usize> {
        byte_offset(source, slice).and_then(|offset| {
            self.chunks
                .iter()
                .position(|chunk| chunk.range.contains(&offset))
        })
    }

    fn has_unmatched(&self) -> bool {
        self.usages.iter().any(|usage| !usage.matched)
    }
}

struct CssBuffer {
    raw: String,
    chunks: Option<Vec<CssChunk>>,
}

impl CssBuffer {
    fn new(track_chunks: bool) -> Self {
        CssBuffer {
            raw: String::new(),
            chunks: track_chunks.then(Vec::new),
        }
    }

    fn push(&mut self, origin: Option<CssOrigin>, content: &str, append_newline: bool) {
        if content.is_empty() {
            return;
        }
        let start = self.raw.len();
        self.raw.push_str(content);
        if append_newline {
            self.raw.push('\n');
        }
        if let (Some(chunks), Some(origin)) = (&mut self.chunks, origin) {
            let end = self.raw.len();
            chunks.push(CssChunk {
                range: start..end,
                origin,
            });
        }
    }

    fn into_parts(self) -> (String, Option<Vec<CssChunk>>) {
        (self.raw, self.chunks)
    }
}

fn byte_offset(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    let hay_ptr = haystack.as_ptr();
    let needle_ptr = needle.as_ptr();
    if needle_ptr < hay_ptr {
        return None;
    }
    // SAFETY: Both pointers originate from the same allocation (`haystack`).
    let diff = unsafe { needle_ptr.offset_from(hay_ptr) };
    if diff.is_negative() {
        None
    } else {
        Some(diff as usize)
    }
}

fn apply_selector_cleanup<'i>(
    state: &SelectorCleanupState<'i>,
    document: &mut Document,
    requested_keep_style_tags: bool,
    declarations: &[parser::Declaration<'i>],
) {
    if state.usages.is_empty() || state.chunks.is_empty() {
        return;
    }
    rewrite_style_blocks(
        state,
        document,
        requested_keep_style_tags,
        declarations,
    );
}

fn rewrite_style_blocks<'i>(
    state: &SelectorCleanupState<'i>,
    document: &mut Document,
    requested_keep_style_tags: bool,
    declarations: &[parser::Declaration<'i>],
) {
    let mut chunk_remainders: Vec<Vec<RuleRemainder<'i>>> = (0..state.chunks.len())
        .map(|_| Vec::new())
        .collect();
    let mut remainder_lookup: FxHashMap<(usize, usize), usize> = FxHashMap::default();

    for usage in &state.usages {
        if usage.matched {
            continue;
        }
        let Some(selector) = usage.selectors else {
            continue;
        };
        let trimmed = selector.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = (usage.chunk_index, usage.rule_id);
        let entry_index = remainder_lookup.entry(key).or_insert_with(|| {
            let idx = chunk_remainders[usage.chunk_index].len();
            chunk_remainders[usage.chunk_index].push(RuleRemainder {
                selectors: SelectorList::new(),
                declarations: usage.declarations,
            });
            idx
        });
        chunk_remainders[usage.chunk_index][*entry_index]
            .selectors
            .push(trimmed);
    }

    for (idx, chunk) in state.chunks.iter().enumerate() {
        let Some(rules) = chunk_remainders.get_mut(idx) else {
            continue;
        };
        if rules.is_empty() {
            handle_empty_remainder(document, chunk, requested_keep_style_tags);
            continue;
        }
        let mut buffer = String::new();
        for remainder in rules.iter() {
            if remainder.selectors.is_empty() {
                continue;
            }
            append_rule(&mut buffer, remainder, declarations);
        }
        if buffer.trim().is_empty() {
            handle_empty_remainder(document, chunk, requested_keep_style_tags);
            continue;
        }
        match chunk.origin {
            CssOrigin::StyleNode(node_id) => {
                overwrite_style_node(document, node_id, buffer.trim_end());
            }
            _ => {
                // Non-inline chunks cannot be rewritten safely yet.
            }
        }
    }
}

fn handle_empty_remainder(
    document: &mut Document,
    chunk: &CssChunk,
    requested_keep_style_tags: bool,
) {
    if let CssOrigin::StyleNode(node_id) = chunk.origin {
        if requested_keep_style_tags {
            overwrite_style_node(document, node_id, "");
        } else {
            document.detach_node(node_id);
        }
    }
}


fn append_rule<'i>(
    buffer: &mut String,
    remainder: &RuleRemainder<'i>,
    declarations: &[parser::Declaration<'i>],
) {
    if remainder.selectors.is_empty() {
        return;
    }
    let (start, end) = remainder.declarations;
    if start >= end || end > declarations.len() {
        return;
    }
    let mut selectors_iter = remainder.selectors.iter().peekable();
    while let Some(selector) = selectors_iter.next() {
        buffer.push_str(selector);
        if selectors_iter.peek().is_some() {
            buffer.push_str(", ");
        }
    }
    buffer.push_str(" {");
    for (name, value) in &declarations[start..end] {
        buffer.push(' ');
        buffer.push_str(name);
        buffer.push(':');
        buffer.push(' ');
        let value_trimmed = value.trim();
        buffer.push_str(value_trimmed);
        if !value_trimmed.ends_with(';') {
            buffer.push(';');
        }
    }
    buffer.push_str(" }\n");
}

fn overwrite_style_node(document: &mut Document, node_id: NodeId, new_css: &str) {
    let new_css = new_css.trim();
    let text_node_id = document[node_id].first_child;
    if let Some(text_node_id) = text_node_id {
        if let NodeData::Text { text } = &mut document[text_node_id].data {
            text.clear();
            text.push_slice(new_css);
            return;
        }
    }
    if new_css.is_empty() {
        return;
    }
    let text_node = document.new_text_node(StrTendril::from(new_css));
    document.append_child(node_id, text_node);
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

    /// Override whether "at-rules" should be kept after processing.
    #[must_use]
    pub fn keep_at_rules(mut self, keep_at_rules: bool) -> Self {
        self.keep_at_rules = keep_at_rules;
        self
    }

    /// Override whether trailing semicolons and spaces between properties and values should be removed.
    #[must_use]
    pub fn minify_css(mut self, minify_css: bool) -> Self {
        self.minify_css = minify_css;
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

    /// Remove selectors that were successfully inlined from inline `<style>` blocks.
    #[must_use]
    pub fn remove_inlined_selectors(mut self, enabled: bool) -> Self {
        self.remove_inlined_selectors = enabled;
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
            keep_at_rules: false,
            minify_css: false,
            base_url: None,
            load_remote_stylesheets: true,
            #[cfg(feature = "stylesheet-cache")]
            cache: None,
            extra_css: None,
            preallocate_node_capacity: 32,
            resolver: Arc::new(DefaultStylesheetResolver),
            remove_inlined_selectors: false,
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
        let mut document = Document::parse_with_options(
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
        let track_selector_cleanup = self.options.remove_inlined_selectors;
        let mut size_estimate: usize = if self.options.inline_style_tags {
            document
                .styles()
                .map(|(_, s)| {
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
        let mut css_buffer = CssBuffer::new(track_selector_cleanup);
        css_buffer.raw.reserve(size_estimate);
        if self.options.inline_style_tags || self.options.keep_at_rules {
            for (node_id, style) in document.styles() {
                let origin = track_selector_cleanup.then(|| CssOrigin::StyleNode(node_id));
                css_buffer.push(origin, style, true);
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
                        let origin =
                            track_selector_cleanup.then_some(CssOrigin::LinkedStylesheet);
                        css_buffer.push(origin, cached, true);
                        continue;
                    }
                }

                let css = self.options.resolver.retrieve(url.as_ref())?;
                let origin = track_selector_cleanup.then_some(CssOrigin::LinkedStylesheet);
                css_buffer.push(origin, &css, true);

                #[cfg(feature = "stylesheet-cache")]
                if let Some(lock) = self.options.cache.as_ref() {
                    let mut cache = lock.lock().expect("Cache lock is poisoned");
                    cache.put(url.into_owned(), css);
                }
            }
        }
        if let Some(extra_css) = &self.options.extra_css {
            let origin = track_selector_cleanup.then_some(CssOrigin::ExtraCss);
            css_buffer.push(origin, extra_css, false);
        }
        if let Some(css) = css {
            let origin = track_selector_cleanup.then_some(CssOrigin::FragmentCss);
            css_buffer.push(origin, css, false);
        }
        let (raw_styles, css_chunks) = css_buffer.into_parts();
        let mut selector_cleanup_state = SelectorCleanupState::new(track_selector_cleanup);
        if let (Some(state), Some(chunks)) = (&mut selector_cleanup_state, css_chunks) {
            state.chunks = chunks;
        }
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
        let at_rules = if self.options.keep_at_rules {
            let mut at_rules = String::new();
            for rule in cssparser::StyleSheetParser::new(
                &mut parser,
                &mut parser::AtRuleFilteringParser::new(&mut declarations, &mut at_rules),
            )
            .flatten()
            {
                if self.options.inline_style_tags {
                    rule_list.push(rule);
                }
            }
            Some(at_rules)
        } else if !raw_styles.is_empty() {
            // At this point, we collected some styles from at least one source, hence we need to process it.
            for rule in cssparser::StyleSheetParser::new(
                &mut parser,
                &mut parser::CSSRuleListParser::new(&mut declarations),
            )
            .flatten()
            {
                rule_list.push(rule);
            }
            None
        } else {
            None
        };
        let mut styles = IndexMap::with_capacity_and_hasher(
            document.elements.len().max(16),
            BuildNoHashHasher::default(),
        );
        // This cache is unused but required in the `selectors` API
        let mut caches = SelectorCaches::default();
        for (rule_id, (selectors, (start, end))) in rule_list.iter().enumerate() {
            // Only CSS Syntax Level 3 is supported, therefore it is OK to split by `,`
            // With `is` or `where` selectors (Level 4) this split should be done on the parser level
            for selector in selectors.split(',') {
                let chunk_index = selector_cleanup_state
                    .as_ref()
                    .and_then(|state| state.chunk_index_for_slice(selectors, &raw_styles));
                let mut usage = chunk_index.map(|chunk_idx| SelectorUsage {
                    selectors: Some(selector),
                    declarations: (*start, *end),
                    rule_id,
                    chunk_index: chunk_idx,
                    ..SelectorUsage::default()
                });
                let mut matched_any = false;
                if let Ok(matching_elements) = document.select(selector, &mut caches) {
                    let specificity = matching_elements.specificity();
                    for matching_element in matching_elements {
                        matched_any = true;
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
                                        value.trim_end().ends_with("!important"),
                                        entry.get().1.trim_end().ends_with("!important"),
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
                if let (Some(state), Some(mut record)) =
                    (selector_cleanup_state.as_mut(), usage.take())
                {
                    record.matched = matched_any;
                    state.record_usage(record);
                }
                // Ignore not parsable selectors. E.g. there is no parser for @media queries
                // Which means that they will fall into this category and will be ignored
            }
        }
        let cleanup_requires_css = selector_cleanup_state
            .as_ref()
            .is_some_and(SelectorCleanupState::has_unmatched);
        let keep_style_tags = self.options.keep_style_tags || cleanup_requires_css;
        if let Some(state) = selector_cleanup_state.as_ref() {
            apply_selector_cleanup(
                state,
                &mut document,
                self.options.keep_style_tags,
                &declarations,
            );
        }
        document.serialize(
            target,
            styles,
            keep_style_tags,
            self.options.keep_link_tags,
            self.options.minify_css,
            at_rules.as_ref(),
            mode,
        )?;
        Ok(())
    }

    fn get_full_url<'u>(&self, href: &'u str) -> Cow<'u, str> {
        // Valid absolute URL
        if Url::parse(href).is_ok() {
            return Cow::Borrowed(href);
        }
        if let Some(base_url) = &self.options.base_url {
            // Use the same scheme as the base URL
            if href.starts_with("//") {
                return Cow::Owned(format!("{}:{}", base_url.scheme(), href));
            }
            // Not a URL, then it is a relative URL
            if let Ok(new_url) = base_url.join(href) {
                return Cow::Owned(new_url.into());
            }
        }
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
