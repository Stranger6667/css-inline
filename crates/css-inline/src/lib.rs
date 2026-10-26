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
mod html;
mod parser;
mod resolver;

pub use error::InlineError;
#[cfg(feature = "stylesheet-cache")]
use lru::{DefaultHasher, LruCache};
use selectors::context::SelectorCaches;
use smallvec::SmallVec;
use std::{borrow::Cow, fmt::Formatter, io::Write, iter::Peekable, ops::Range, sync::Arc};

#[cfg(feature = "async")]
pub use async_trait::async_trait;
use html::{Document, InliningMode, NodeData, NodeId, Precedence};
use parser::{LayerId, Layers, Rule, Span};
pub use resolver::{DefaultStylesheetResolver, StylesheetResolver};
use rustc_hash::FxHashMap;
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
    /// Apply `width` HTML attributes from CSS `width` properties on supported elements.
    ///
    /// This is useful for email compatibility with clients like Outlook that ignore CSS width.
    /// Supported elements: `table`, `td`, `th`, `img`.
    pub apply_width_attributes: bool,
    /// Apply `height` HTML attributes from CSS `height` properties on supported elements.
    ///
    /// This is useful for email compatibility with clients like Outlook that ignore CSS height.
    /// Supported elements: `table`, `td`, `th`, `img`.
    pub apply_height_attributes: bool,
    /// Substitute `var()` references with the values of the custom properties they refer to
    /// and drop the `--*` declarations from the inlined styles.
    ///
    /// Email clients do not support custom properties; this makes the output work without them.
    pub resolve_css_variables: bool,
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
            .field("apply_width_attributes", &self.apply_width_attributes)
            .field("apply_height_attributes", &self.apply_height_attributes)
            .field("resolve_css_variables", &self.resolve_css_variables)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct CssChunk {
    range: Range<usize>,
    /// The style node this chunk came from, if any.
    /// `None` for linked stylesheets, extra CSS, or fragment CSS.
    style_node: Option<NodeId>,
}

type SelectorList<'i> = SmallVec<[&'i str; 2]>;

#[derive(Debug)]
struct SelectorUsage<'i> {
    selector: &'i str,
    declarations: (usize, usize),
    rule_id: usize,
    chunk_index: usize,
    matched: bool,
}

#[derive(Debug, Default)]
struct RuleRemainder<'i> {
    selectors: SelectorList<'i>,
    declarations: (usize, usize),
    /// Offset of the first selector in the combined CSS.
    offset: usize,
}

#[derive(Debug, Default)]
struct SelectorCleanupState<'i> {
    chunks: Vec<CssChunk>,
    usages: Vec<SelectorUsage<'i>>,
}

impl<'i> SelectorCleanupState<'i> {
    fn record_usage(&mut self, usage: SelectorUsage<'i>) {
        self.usages.push(usage);
    }
}

/// A kept `@layer` block; the range covers its contents.
struct LayerBlock<'i> {
    range: Range<usize>,
    prelude: &'i str,
    layer: LayerId,
    created: bool,
}

/// Renders the `@`-rules and leftover style rules that stay in the output, wrapped in their
/// `@layer` blocks.
struct KeptCss<'a, 'i> {
    source: &'i str,
    layers: &'a Layers<'i>,
    /// Layers whose rules stay in the output, with their ancestors.
    kept: Vec<bool>,
}

impl<'a, 'i> KeptCss<'a, 'i> {
    /// Rules stay in the output as kept `@`-rules, as selector cleanup leftovers, or verbatim
    /// when the `<style>` tags are kept as they are.
    fn new(
        source: &'i str,
        layers: &'a Layers<'i>,
        spans: &[Span<'i>],
        rules: &[Rule<'i>],
        cleanup: Option<&SelectorCleanupState<'i>>,
        verbatim_styles: bool,
    ) -> Self {
        let mut kept = vec![false; layers.len()];
        let mut mark = |mut id: LayerId| {
            while id != 0 && !kept[id] {
                kept[id] = true;
                id = layers.parent(id);
            }
        };
        for span in spans {
            if let Span::AtRule { layer, .. } = span {
                mark(*layer);
            }
        }
        if let Some(state) = cleanup {
            for usage in state.usages.iter().filter(|usage| !usage.matched) {
                mark(rules[usage.rule_id].layer);
            }
        }
        if verbatim_styles {
            for rule in rules {
                mark(rule.layer);
            }
        }
        KeptCss {
            source,
            layers,
            kept,
        }
    }

    /// Whether `layer` or an ancestor has rules in the output, so its position matters.
    fn is_kept(&self, mut layer: LayerId) -> bool {
        while layer != 0 {
            if self.kept[layer] {
                return true;
            }
            layer = self.layers.parent(layer);
        }
        false
    }

    /// Render the spans within `range` of the source together with `items` (offset, text),
    /// `sep`-separated within `@layer` blocks. `@`-rules starting before `at_rules_from` are
    /// skipped, while the layer structure around them stays.
    fn render(
        &self,
        spans: &[Span<'i>],
        range: &Range<usize>,
        at_rules_from: usize,
        mut items: Vec<(usize, String)>,
        sep: &str,
    ) -> Vec<String> {
        let mut blocks = Vec::new();
        for span in spans.iter().filter(|span| range.contains(&span.start())) {
            match span {
                Span::AtRule { range: at_rule, .. } if at_rule.start < at_rules_from => {}
                // An unterminated `@`-rule runs into the following blocks; it ends with its own.
                Span::AtRule { range: at_rule, .. } => items.push((
                    at_rule.start,
                    self.source[at_rule.start..at_rule.end.min(range.end)]
                        .trim()
                        .to_string(),
                )),
                Span::LayerStatement {
                    start,
                    prelude,
                    layers,
                } => {
                    if layers.iter().any(|&layer| self.is_kept(layer)) {
                        items.push((*start, format!("@layer {};", prelude.trim())));
                    }
                }
                Span::LayerBlock {
                    range: block,
                    prelude,
                    layer,
                    created,
                } => blocks.push(LayerBlock {
                    range: block.start..block.end.min(range.end),
                    prelude,
                    layer: *layer,
                    created: *created,
                }),
            }
        }
        items.retain(|(_, text)| !text.is_empty());
        items.sort_by_key(|(offset, _)| *offset);
        blocks.sort_by_key(|block| block.range.start);
        self.render_tree(&mut items.into_iter().peekable(), &blocks, range.end, sep)
    }

    /// Blocks are sorted by start and nest, so the blocks starting inside one are its children.
    #[allow(clippy::arithmetic_side_effects)]
    fn render_tree(
        &self,
        items: &mut Peekable<std::vec::IntoIter<(usize, String)>>,
        blocks: &[LayerBlock<'i>],
        end: usize,
        sep: &str,
    ) -> Vec<String> {
        let mut out = Vec::new();
        let mut i = 0;
        while let Some(block) = blocks.get(i) {
            while let Some((_, text)) = items.next_if(|(offset, _)| *offset < block.range.start) {
                out.push(text);
            }
            let mut j = i + 1;
            while blocks
                .get(j)
                .is_some_and(|child| child.range.start < block.range.end)
            {
                j += 1;
            }
            let inner = self.render_tree(items, &blocks[i + 1..j], block.range.end, sep);
            let prelude = block.prelude.trim();
            if inner.is_empty() {
                // The block declared the layer; a statement keeps its position.
                if block.created && !prelude.is_empty() && self.is_kept(block.layer) {
                    out.push(format!("@layer {prelude};"));
                }
            } else {
                let inner = inner.join(sep);
                if prelude.is_empty() {
                    out.push(format!("@layer {{{sep}{inner}{sep}}}"));
                } else {
                    out.push(format!("@layer {prelude} {{{sep}{inner}{sep}}}"));
                }
            }
            i = j;
        }
        while let Some((_, text)) = items.next_if(|(offset, _)| *offset < end) {
            out.push(text);
        }
        out
    }
}

/// Find which chunk contains the given byte offset.
fn find_chunk_index(chunks: &[CssChunk], offset: usize) -> Option<usize> {
    chunks
        .iter()
        .position(|chunk| chunk.range.contains(&offset))
}

/// Compute chunk indices for all rules based on where their selectors point in the source.
#[allow(clippy::arithmetic_side_effects)]
fn compute_rule_chunk_indices(
    rules: &[Rule<'_>],
    source: &str,
    chunks: &[CssChunk],
) -> Vec<Option<usize>> {
    let source_start = source.as_ptr() as usize;
    let source_end = source_start.saturating_add(source.len());

    rules
        .iter()
        .map(|rule| {
            let sel_start = rule.selectors.as_ptr() as usize;
            // Check if selectors slice is within source bounds
            if sel_start >= source_start && sel_start < source_end {
                let offset = sel_start.wrapping_sub(source_start);
                find_chunk_index(chunks, offset)
            } else {
                None
            }
        })
        .collect()
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

    fn push(&mut self, style_node: Option<NodeId>, content: &str, append_newline: bool) {
        if content.is_empty() {
            return;
        }
        let start = self.raw.len();
        self.raw.push_str(content);
        if append_newline {
            self.raw.push('\n');
        }
        if let Some(chunks) = &mut self.chunks {
            let end = self.raw.len();
            chunks.push(CssChunk {
                range: start..end,
                style_node,
            });
        }
    }

    fn into_parts(self) -> (String, Option<Vec<CssChunk>>) {
        (self.raw, self.chunks)
    }
}

/// Rewrite the `<style>` blocks to what remains after inlining; whether any of them keeps content.
fn apply_selector_cleanup<'i>(
    state: &SelectorCleanupState<'i>,
    document: &mut Document,
    requested_keep_style_tags: bool,
    declarations: &[parser::Declaration<'i>],
    kept: &KeptCss<'_, 'i>,
    spans: &[Span<'i>],
) -> bool {
    let source = kept.source;
    let source_start = source.as_ptr() as usize;
    let mut chunk_remainders: Vec<Vec<RuleRemainder<'i>>> =
        (0..state.chunks.len()).map(|_| Vec::new()).collect();
    let mut remainder_lookup: FxHashMap<(usize, usize), usize> = FxHashMap::default();

    for usage in &state.usages {
        if usage.matched {
            continue;
        }
        let trimmed = usage.selector.trim();
        if trimmed.is_empty() {
            continue;
        }
        let key = (usage.chunk_index, usage.rule_id);
        let entry_index = remainder_lookup.entry(key).or_insert_with(|| {
            let idx = chunk_remainders[usage.chunk_index].len();
            chunk_remainders[usage.chunk_index].push(RuleRemainder {
                selectors: SelectorList::new(),
                declarations: usage.declarations,
                offset: (trimmed.as_ptr() as usize).wrapping_sub(source_start),
            });
            idx
        });
        chunk_remainders[usage.chunk_index][*entry_index]
            .selectors
            .push(trimmed);
    }

    let mut keeps_content = false;
    for (idx, chunk) in state.chunks.iter().enumerate() {
        let Some(node_id) = chunk.style_node else {
            continue;
        };
        // Remainders and `@`-rules, keyed by their offset so the block keeps source order.
        let mut items: Vec<(usize, String)> = Vec::with_capacity(chunk_remainders[idx].len());
        for remainder in &chunk_remainders[idx] {
            let mut text = String::new();
            append_rule(&mut text, remainder, declarations);
            items.push((remainder.offset, text));
        }
        let rules = kept.render(spans, &chunk.range, 0, items, "\n");
        if rules.is_empty() {
            handle_empty_remainder(document, chunk, requested_keep_style_tags);
            continue;
        }
        keeps_content = true;
        overwrite_style_node(document, node_id, &rules);
    }
    keeps_content
}

fn handle_empty_remainder(
    document: &mut Document,
    chunk: &CssChunk,
    requested_keep_style_tags: bool,
) {
    if let Some(node_id) = chunk.style_node {
        if requested_keep_style_tags {
            overwrite_style_node(document, node_id, &[]);
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
    let (start, end) = remainder.declarations;
    if start >= end || end > declarations.len() {
        return;
    }
    let declarations = &declarations[start..end];
    // `selector, ` per selector, ` name: value;` per declaration and the braces.
    let size = remainder
        .selectors
        .iter()
        .map(|selector| selector.len().saturating_add(2))
        .chain(
            declarations
                .iter()
                .map(|(name, value)| name.len().saturating_add(value.len()).saturating_add(4)),
        )
        .fold(2, usize::saturating_add);
    buffer.reserve(size);
    let mut selectors_iter = remainder.selectors.iter().peekable();
    while let Some(selector) = selectors_iter.next() {
        buffer.push_str(selector);
        if selectors_iter.peek().is_some() {
            buffer.push_str(", ");
        }
    }
    buffer.push_str(" {");
    for (name, value) in declarations {
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
    buffer.push_str(" }");
}

/// Replace the text of a `<style>` node with `rules` joined by newlines.
fn overwrite_style_node(document: &mut Document, node_id: NodeId, rules: &[String]) {
    if let Some(text_node_id) = document[node_id].first_child {
        if let NodeData::Text { text } = &mut document[text_node_id].data {
            text.clear();
            for (idx, rule) in rules.iter().enumerate() {
                // Every rendered rule is non-empty and trimmed, so the joined text needs no trim.
                debug_assert!(!rule.is_empty() && rule.trim().len() == rule.len());
                if idx != 0 {
                    text.push_char('\n');
                }
                text.push_slice(rule);
            }
        }
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

    /// Apply `width` HTML attributes from CSS `width` properties on supported elements.
    ///
    /// This is useful for email compatibility with clients like Outlook that ignore CSS width.
    /// Supported elements: `table`, `td`, `th`, `img`.
    #[must_use]
    pub fn apply_width_attributes(mut self, apply: bool) -> Self {
        self.apply_width_attributes = apply;
        self
    }

    /// Apply `height` HTML attributes from CSS `height` properties on supported elements.
    ///
    /// This is useful for email compatibility with clients like Outlook that ignore CSS height.
    /// Supported elements: `table`, `td`, `th`, `img`.
    #[must_use]
    pub fn apply_height_attributes(mut self, apply: bool) -> Self {
        self.apply_height_attributes = apply;
        self
    }

    /// Substitute `var()` references with the values of the custom properties they refer to
    /// and drop the `--*` declarations from the inlined styles.
    #[must_use]
    pub fn resolve_css_variables(mut self, resolve: bool) -> Self {
        self.resolve_css_variables = resolve;
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
            apply_width_attributes: false,
            apply_height_attributes: false,
            resolve_css_variables: false,
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

/// Reuse the output buffer instead of copying it into a new `String`.
fn into_string(out: Vec<u8>) -> String {
    String::from_utf8(out)
        .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned())
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
        Ok(into_string(out))
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
        self.inline_to_impl(html, None, target, InliningMode::Document, &|url| {
            self.options.resolver.retrieve(url)
        })
    }

    /// Inline CSS into an HTML fragment.
    ///
    /// Unlike [`inline`](CSSInliner::inline), this method does not wrap the output in `<html>` and
    /// `<body>` tags. Structural tags (`<html>`, `<head>`, `<body>`) are stripped from the output;
    /// only their contents are preserved. CSS from `<style>` tags within the input is also
    /// processed and inlined.
    ///
    /// If you need to preserve the full HTML document structure, use [`inline`](CSSInliner::inline)
    /// instead.
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
        Ok(into_string(out))
    }

    /// Inline CSS into an HTML fragment and write the result to a generic writer.
    ///
    /// See [`inline_fragment`](CSSInliner::inline_fragment) for details on fragment handling.
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
        self.inline_to_impl(html, Some(css), target, InliningMode::Fragment, &|url| {
            self.options.resolver.retrieve(url)
        })
    }

    /// Non-blocking version of [`CSSInliner::inline`]. External stylesheets are fetched concurrently
    /// via [`StylesheetResolver::retrieve_async`].
    ///
    /// # Errors
    ///
    /// Same as [`CSSInliner::inline`].
    ///
    /// # Panics
    ///
    /// If the external stylesheet cache lock is poisoned.
    #[cfg(feature = "async")]
    pub async fn inline_async(&self, html: &str) -> Result<String> {
        let mut out = allocate_output_buffer(html);
        self.inline_to_async(html, &mut out).await?;
        Ok(into_string(out))
    }

    /// Non-blocking version of [`CSSInliner::inline_to`].
    ///
    /// # Errors
    ///
    /// Same as [`CSSInliner::inline_to`].
    ///
    /// # Panics
    ///
    /// If the external stylesheet cache lock is poisoned.
    #[cfg(feature = "async")]
    pub async fn inline_to_async<W: Write>(&self, html: &str, target: &mut W) -> Result<()> {
        let stylesheets = self.fetch_stylesheets(html, InliningMode::Document).await?;
        self.inline_to_impl(html, None, target, InliningMode::Document, &|url| {
            // `fetch_stylesheets` collects the same URLs from the same HTML
            Ok(stylesheets[url].clone())
        })
    }

    /// Non-blocking version of [`CSSInliner::inline_fragment`].
    ///
    /// # Errors
    ///
    /// Same as [`CSSInliner::inline_fragment`].
    ///
    /// # Panics
    ///
    /// If the external stylesheet cache lock is poisoned.
    #[cfg(feature = "async")]
    pub async fn inline_fragment_async(&self, html: &str, css: &str) -> Result<String> {
        let mut out = allocate_output_buffer(html);
        self.inline_fragment_to_async(html, css, &mut out).await?;
        Ok(into_string(out))
    }

    /// Non-blocking version of [`CSSInliner::inline_fragment_to`].
    ///
    /// # Errors
    ///
    /// Same as [`CSSInliner::inline_fragment_to`].
    ///
    /// # Panics
    ///
    /// If the external stylesheet cache lock is poisoned.
    #[cfg(feature = "async")]
    pub async fn inline_fragment_to_async<W: Write>(
        &self,
        html: &str,
        css: &str,
        target: &mut W,
    ) -> Result<()> {
        let stylesheets = self.fetch_stylesheets(html, InliningMode::Fragment).await?;
        self.inline_to_impl(html, Some(css), target, InliningMode::Fragment, &|url| {
            // `fetch_stylesheets` collects the same URLs from the same HTML
            Ok(stylesheets[url].clone())
        })
    }

    #[cfg(feature = "async")]
    async fn fetch_stylesheets(
        &self,
        html: &str,
        mode: InliningMode,
    ) -> Result<FxHashMap<String, String>> {
        let mut stylesheets = FxHashMap::default();
        if !self.options.load_remote_stylesheets {
            return Ok(stylesheets);
        }
        let mut missing = Vec::new();
        {
            // Dropped before any `.await`, as `Document` is not `Send`
            let document =
                Document::parse_with_options(html, self.options.preallocate_node_capacity, mode);
            let mut urls = document
                .stylesheets()
                .map(|href| self.get_full_url(href).into_owned())
                .collect::<Vec<_>>();
            urls.sort_unstable();
            urls.dedup();
            for url in urls {
                // Copied, so an eviction before `inline_to_impl` reads the cache does not lose it
                #[cfg(feature = "stylesheet-cache")]
                if let Some(lock) = self.options.cache.as_ref() {
                    let mut cache = lock.lock().expect("Cache lock is poisoned");
                    if let Some(css) = cache.get(&url) {
                        stylesheets.insert(url, css.clone());
                        continue;
                    }
                }
                missing.push(url);
            }
        }
        let fetched = futures_util::future::try_join_all(
            missing
                .iter()
                .map(|url| self.options.resolver.retrieve_async(url)),
        )
        .await?;
        stylesheets.extend(missing.into_iter().zip(fetched));
        Ok(stylesheets)
    }

    #[allow(clippy::too_many_lines)]
    fn inline_to_impl<W: Write>(
        &self,
        html: &str,
        css: Option<&str>,
        target: &mut W,
        mode: InliningMode,
        retrieve: &dyn Fn(&str) -> Result<String>,
    ) -> Result<()> {
        let mut document =
            Document::parse_with_options(html, self.options.preallocate_node_capacity, mode);
        // CSS rules may overlap, and the final set of rules applied to an element depend on
        // selectors' specificity - selectors with higher specificity have more priority.
        // Inlining happens in two major steps:
        //   1. All available styles are mapped to respective elements together with their
        //      selector's specificity. When two rules overlap on the same declaration, then
        //      the one with higher specificity replaces another.
        //   2. Resulting styles are merged into existing "style" tags.
        // Without `inline_style_tags` no `<style>` rule is inlined, so none is removed.
        let track_selector_cleanup =
            self.options.remove_inlined_selectors && self.options.inline_style_tags;
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
                let style_node = track_selector_cleanup.then_some(node_id);
                css_buffer.push(style_node, style, true);
            }
        }
        let styles_end = css_buffer.raw.len();
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
                        css_buffer.push(None, cached, true);
                        continue;
                    }
                }

                let css = retrieve(url.as_ref())?;
                css_buffer.push(None, &css, true);

                #[cfg(feature = "stylesheet-cache")]
                if let Some(lock) = self.options.cache.as_ref() {
                    let mut cache = lock.lock().expect("Cache lock is poisoned");
                    cache.put(url.into_owned(), css);
                }
            }
        }
        if let Some(extra_css) = &self.options.extra_css {
            css_buffer.push(None, extra_css, false);
        }
        if let Some(css) = css {
            css_buffer.push(None, css, false);
        }
        let (raw_styles, css_chunks) = css_buffer.into_parts();
        let mut selector_cleanup_state = if track_selector_cleanup {
            Some(SelectorCleanupState::default())
        } else {
            None
        };
        if let (Some(state), Some(chunks)) = (&mut selector_cleanup_state, css_chunks) {
            state.chunks = chunks;
        }
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
        // `@`-rules are not inlined. `keep_at_rules` moves them to a new `<style>` and selector
        // cleanup keeps them in the rewritten ones, so both need their spans.
        let mut spans = (self.options.keep_at_rules || track_selector_cleanup).then(Vec::new);
        let mut rule_parser = parser::RuleParser::new(
            &raw_styles,
            &mut declarations,
            &mut rule_list,
            spans.as_mut(),
        );
        if !raw_styles.is_empty() {
            let mut parser = cssparser::Parser::new(&raw_styles);
            let mut items = cssparser::StyleSheetParser::new(&mut parser, &mut rule_parser);
            parser::collect_rules(&mut items);
        }
        let layers = rule_parser.into_layers();
        let spans = spans.unwrap_or_default();
        if self.options.keep_at_rules && !self.options.inline_style_tags {
            // The styles were collected only for their `@`-rules.
            rule_list.clear();
        }
        let ranks = layers.ranks();
        // Important declarations cascade in reverse layer order.
        let last_rank = u32::try_from(layers.len().saturating_sub(1)).unwrap_or(u32::MAX);
        let reversed = |precedence: Precedence| {
            Precedence(last_rank.saturating_sub(precedence.0), precedence.1)
        };
        // Compute chunk indices for all rules once, before processing
        let rule_chunk_indices = selector_cleanup_state
            .as_ref()
            .map(|state| compute_rule_chunk_indices(&rule_list, &raw_styles, &state.chunks))
            .unwrap_or_default();
        // Vec indexed by NodeId for O(1) access instead of hash lookups
        let mut styles: Vec<Option<html::ElementStyleMap<'_>>> = vec![None; document.nodes.len()];
        // This cache is unused but required in the `selectors` API
        let mut caches = SelectorCaches::default();
        for (rule_id, rule) in rule_list.iter().enumerate() {
            let (start, end) = rule.declarations;
            let rank = ranks[rule.layer];
            for selector in parser::split_selector_list(rule.selectors) {
                let mut matched_any = false;
                // Quick check: skip selectors whose anchor doesn't exist in the document
                // This avoids parsing selectors that can't possibly match anything
                if !document.anchor_exists(selector) {
                    if let Some(state) = selector_cleanup_state.as_mut() {
                        if let Some(chunk_index) =
                            rule_chunk_indices.get(rule_id).copied().flatten()
                        {
                            state.record_usage(SelectorUsage {
                                selector,
                                declarations: (start, end),
                                rule_id,
                                chunk_index,
                                matched: false,
                            });
                        }
                    }
                    continue;
                }
                if let Ok(matching_elements) = document.select(selector, &mut caches) {
                    let precedence = Precedence(rank, matching_elements.specificity());
                    for matching_element in matching_elements {
                        matched_any = true;
                        let element_styles = styles[matching_element.node_id.get()]
                            .get_or_insert_with(SmallVec::new);
                        // Iterate over pairs of property name & value
                        // Example: `padding`, `0`
                        for (name, value) in &declarations[start..end] {
                            let prop_name = name.as_ref();
                            // Linear search for existing property
                            if let Some(idx) =
                                element_styles.iter().position(|(n, _, _)| *n == prop_name)
                            {
                                let entry = &mut element_styles[idx];
                                let new_important = parser::strip_important(value).is_some();
                                let old_important = parser::strip_important(entry.2).is_some();
                                match (new_important, old_important) {
                                    // Both normal; the later layer, then the higher specificity wins.
                                    (false, false) => {
                                        if entry.1 <= precedence {
                                            entry.1 = precedence;
                                            entry.2 = *value;
                                        }
                                    }
                                    // Both important; the earlier layer, then the higher specificity wins.
                                    (true, true) => {
                                        if reversed(entry.1) <= reversed(precedence) {
                                            entry.1 = precedence;
                                            entry.2 = *value;
                                        }
                                    }
                                    // Only the new value is important; it wins.
                                    (true, false) => {
                                        entry.1 = precedence;
                                        entry.2 = *value;
                                    }
                                    // The old value is important and the new one is not; keep
                                    // the old value.
                                    (false, true) => {}
                                }
                            } else {
                                element_styles.push((prop_name, precedence, *value));
                            }
                        }
                    }
                }
                if let Some(state) = selector_cleanup_state.as_mut() {
                    if let Some(chunk_index) = rule_chunk_indices.get(rule_id).copied().flatten() {
                        state.record_usage(SelectorUsage {
                            selector,
                            declarations: (start, end),
                            rule_id,
                            chunk_index,
                            matched: matched_any,
                        });
                    }
                }
                // Ignore not parsable selectors. E.g. there is no parser for @media queries
                // Which means that they will fall into this category and will be ignored
            }
        }
        let mut keep_style_tags = self.options.keep_style_tags;
        let mut at_rules = None;
        if self.options.keep_at_rules || selector_cleanup_state.is_some() {
            let kept = KeptCss::new(
                &raw_styles,
                &layers,
                &spans,
                &rule_list,
                selector_cleanup_state.as_ref(),
                self.options.keep_style_tags && !track_selector_cleanup,
            );
            if self.options.keep_at_rules {
                // `<style>` `@`-rules stay in their blocks when the blocks are kept or rewritten;
                // only the rest (remote stylesheets, `extra_css`) goes to the new `<style>`, with
                // the layer order declared before it.
                let at_rules_from = if self.options.keep_style_tags || track_selector_cleanup {
                    styles_end
                } else {
                    0
                };
                let has_at_rules = at_rules_from == 0
                    || spans.iter().any(|span| {
                        !matches!(span, Span::LayerBlock { .. }) && span.start() >= at_rules_from
                    });
                if has_at_rules {
                    let rules = kept.render(
                        &spans,
                        &(0..raw_styles.len()),
                        at_rules_from,
                        Vec::new(),
                        " ",
                    );
                    at_rules = (!rules.is_empty()).then(|| format!("{} ", rules.join(" ")));
                }
            }
            if let Some(state) = selector_cleanup_state.as_ref() {
                keep_style_tags |= apply_selector_cleanup(
                    state,
                    &mut document,
                    self.options.keep_style_tags,
                    &declarations,
                    &kept,
                    &spans,
                );
            }
        }
        document.serialize(
            target,
            styles,
            keep_style_tags,
            self.options.keep_link_tags,
            self.options.minify_css,
            at_rules.as_ref(),
            mode,
            self.options.apply_width_attributes,
            self.options.apply_height_attributes,
            self.options.resolve_css_variables,
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
/// See [`CSSInliner::inline_fragment`] for details on fragment handling.
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

/// Shortcut for non-blocking inlining with default parameters. See [`CSSInliner::inline_async`].
///
/// # Errors
///
/// Same as [`CSSInliner::inline`].
#[cfg(feature = "async")]
pub async fn inline_async(html: &str) -> Result<String> {
    CSSInliner::default().inline_async(html).await
}

/// Shortcut for non-blocking inlining with default parameters, writing the output to a generic writer.
///
/// # Errors
///
/// Same as [`CSSInliner::inline`].
#[cfg(feature = "async")]
pub async fn inline_to_async<W: Write>(html: &str, target: &mut W) -> Result<()> {
    CSSInliner::default().inline_to_async(html, target).await
}

/// Shortcut for non-blocking inlining into an HTML fragment with default parameters.
///
/// # Errors
///
/// Same as [`CSSInliner::inline_fragment`].
#[cfg(feature = "async")]
pub async fn inline_fragment_async(html: &str, css: &str) -> Result<String> {
    CSSInliner::default().inline_fragment_async(html, css).await
}

/// Shortcut for non-blocking inlining into an HTML fragment with default parameters, writing the output to a generic writer.
///
/// # Errors
///
/// Same as [`CSSInliner::inline_fragment`].
#[cfg(feature = "async")]
pub async fn inline_fragment_to_async<W: Write>(
    html: &str,
    css: &str,
    target: &mut W,
) -> Result<()> {
    CSSInliner::default()
        .inline_fragment_to_async(html, css, target)
        .await
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
