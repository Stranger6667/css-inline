use cssparser::{ParserState, Token};
use std::ops::Range;

pub(crate) struct CSSDeclarationListParser;

pub(crate) type Name<'i> = cssparser::CowRcStr<'i>;
pub(crate) type Declaration<'i> = (Name<'i>, &'i str);

/// A style rule: its selector list, the range of its declarations and the layer it belongs to.
#[derive(Debug)]
pub(crate) struct Rule<'i> {
    pub(crate) selectors: &'i str,
    pub(crate) declarations: (usize, usize),
    pub(crate) layer: LayerId,
}

/// Split a selector list on its top-level commas, leaving commas inside `()`, `[]`, strings,
/// comments and escapes intact.
pub(crate) fn split_selector_list(list: &str) -> impl Iterator<Item = &str> {
    let mut rest = Some(list);
    std::iter::from_fn(move || {
        let current = rest?;
        if let Some(comma) = top_level_comma(current) {
            let (selector, tail) = current.split_at(comma);
            rest = tail.strip_prefix(',');
            Some(selector)
        } else {
            rest = None;
            Some(current)
        }
    })
}

#[derive(Clone, Copy)]
enum Scan {
    Code,
    String(u8),
    Comment,
}

fn top_level_comma(list: &str) -> Option<usize> {
    let mut scan = Scan::Code;
    let mut depth = 0usize;
    let mut escaped = false;
    let mut previous = 0u8;
    for (idx, &byte) in list.as_bytes().iter().enumerate() {
        // Every special byte is ASCII, so an escape never hides one inside a multibyte char
        if escaped {
            escaped = false;
            previous = 0;
            continue;
        }
        match scan {
            Scan::Comment => {
                if previous == b'*' && byte == b'/' {
                    scan = Scan::Code;
                    previous = 0;
                    continue;
                }
            }
            Scan::String(quote) => {
                if byte == b'\\' {
                    escaped = true;
                } else if byte == quote {
                    scan = Scan::Code;
                }
            }
            Scan::Code => match byte {
                b'\\' => escaped = true,
                b'"' | b'\'' => scan = Scan::String(byte),
                b'*' if previous == b'/' => {
                    scan = Scan::Comment;
                    previous = 0;
                    continue;
                }
                b'(' | b'[' => depth = depth.saturating_add(1),
                b')' | b']' => depth = depth.saturating_sub(1),
                b',' if depth == 0 => return Some(idx),
                _ => {}
            },
        }
        previous = byte;
    }
    None
}

/// Index into [`Layers`]; `0` is the implicit layer of unlayered rules.
pub(crate) type LayerId = usize;

#[derive(Debug)]
struct Layer<'i> {
    parent: LayerId,
    /// Last segment of the layer name; `None` for the root and anonymous layers.
    name: Option<&'i str>,
}

/// Cascade layers in order of first appearance; children always follow their parent.
#[derive(Debug)]
pub(crate) struct Layers<'i>(Vec<Layer<'i>>);

impl<'i> Layers<'i> {
    fn new() -> Self {
        Layers(vec![Layer {
            parent: 0,
            name: None,
        }])
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn parent(&self, id: LayerId) -> LayerId {
        self.0[id].parent
    }

    /// Find or create the layer `name` (possibly dotted) within `parent`.
    /// Returns its id and whether any part of it is new.
    fn register(&mut self, parent: LayerId, name: &'i str) -> (LayerId, bool) {
        let mut id = parent;
        let mut created = false;
        for segment in name.split('.') {
            let child = self
                .0
                .iter()
                .position(|layer| layer.parent == id && layer.name == Some(segment));
            id = child.unwrap_or_else(|| {
                created = true;
                self.push(id, Some(segment))
            });
        }
        (id, created)
    }

    fn push(&mut self, parent: LayerId, name: Option<&'i str>) -> LayerId {
        self.0.push(Layer { parent, name });
        self.0.len().saturating_sub(1)
    }

    /// Rank layers in post-order of the layer tree: sublayers first, then the layer's own
    /// rules; the unlayered root comes last. Normal declarations in higher-ranked layers win,
    /// important ones in lower-ranked layers.
    #[allow(clippy::arithmetic_side_effects)]
    pub(crate) fn ranks(&self) -> Vec<u32> {
        let mut children: Vec<Vec<LayerId>> = vec![Vec::new(); self.0.len()];
        for (id, layer) in self.0.iter().enumerate().skip(1) {
            children[layer.parent].push(id);
        }
        let mut ranks = vec![0; self.0.len()];
        let mut rank = 0;
        let mut stack = vec![(0, 0)];
        while let Some((id, next)) = stack.last_mut() {
            if let Some(&child) = children[*id].get(*next) {
                *next += 1;
                stack.push((child, 0));
            } else {
                ranks[*id] = rank;
                rank += 1;
                stack.pop();
            }
        }
        ranks
    }
}

/// `@`-rules that the output keeps, in source offsets of the combined CSS.
#[derive(Debug)]
pub(crate) enum Span<'i> {
    /// Verbatim `@`-rule text and the layer it is in.
    AtRule { range: Range<usize>, layer: LayerId },
    /// `@layer a, b;`
    LayerStatement {
        start: usize,
        prelude: &'i str,
        layers: Vec<LayerId>,
    },
    /// `@layer a { ... }`; the range covers the block contents.
    LayerBlock {
        range: Range<usize>,
        prelude: &'i str,
        layer: LayerId,
        created: bool,
    },
}

impl Span<'_> {
    pub(crate) fn start(&self) -> usize {
        match self {
            Span::AtRule { range, .. } | Span::LayerBlock { range, .. } => range.start,
            Span::LayerStatement { start, .. } => *start,
        }
    }
}

/// Parser for a rule list: style rules are collected with their declarations, `@layer` blocks
/// are parsed recursively, and every other `@`-rule is rejected so the caller sees its span.
pub(crate) struct RuleParser<'d, 'i> {
    source: &'i str,
    declarations: &'d mut Vec<Declaration<'i>>,
    rules: &'d mut Vec<Rule<'i>>,
    spans: Option<&'d mut Vec<Span<'i>>>,
    layers: Layers<'i>,
    /// The layer of the block being parsed.
    layer: LayerId,
    /// Depth of enclosing conditional group rules (`@media` etc.). Their rules are not inlined
    /// and are kept verbatim, but the layers they declare are registered as if the condition
    /// held: css-inline cannot evaluate it.
    skipped: usize,
    /// Whether the source may contain `@layer`; otherwise conditional rules are not walked.
    has_layers: bool,
}

impl<'d, 'i> RuleParser<'d, 'i> {
    #[inline]
    pub(crate) fn new(
        source: &'i str,
        declarations: &'d mut Vec<Declaration<'i>>,
        rules: &'d mut Vec<Rule<'i>>,
        spans: Option<&'d mut Vec<Span<'i>>>,
    ) -> RuleParser<'d, 'i> {
        RuleParser {
            source,
            declarations,
            rules,
            spans,
            layers: Layers::new(),
            layer: 0,
            skipped: 0,
            has_layers: memchr::memchr_iter(b'@', source.as_bytes()).any(|idx| {
                source
                    .as_bytes()
                    .get(idx.saturating_add(1)..idx.saturating_add(6))
                    .is_some_and(|name| name.eq_ignore_ascii_case(b"layer"))
            }),
        }
    }

    pub(crate) fn into_layers(self) -> Layers<'i> {
        self.layers
    }

    /// Keep `span` in the output, unless an enclosing conditional rule is kept as a whole.
    fn record(&mut self, span: Span<'i>) {
        if self.skipped == 0 {
            if let Some(spans) = &mut self.spans {
                spans.push(span);
            }
        }
    }

    /// Keep the rejected `@`-rule `rule` that ends at byte `end` of the source.
    fn record_at_rule(&mut self, rule: &'i str, end: usize) {
        if self.spans.is_some() && rule.starts_with('@') {
            let start = offset_in(self.source, rule);
            self.record(Span::AtRule {
                range: start..end,
                layer: self.layer,
            });
        }
    }
}

fn offset_in(source: &str, slice: &str) -> usize {
    (slice.as_ptr() as usize).wrapping_sub(source.as_ptr() as usize)
}

/// Drive `items` to the end, recording the spans of rejected `@`-rules.
pub(crate) fn collect_rules<'i>(
    items: &mut cssparser::StyleSheetParser<'i, '_, RuleParser<'_, 'i>>,
) {
    // A rejected rule's block stays pending until the parser moves on; skipping
    // whitespace consumes it, so the rule's span ends there.
    // The error carries the rule's start, past any whitespace and comments.
    let mut rejected = None;
    loop {
        if let Some(rule) = rejected.take() {
            items.input.skip_whitespace();
            let end = items.input.position().byte_index();
            items.parser.record_at_rule(rule, end);
        }
        match items.next() {
            None => break,
            Some(Ok(())) => {}
            Some(Err((_, rule, _))) => rejected = Some(rule),
        }
    }
}

/// The declaration value without its `!important` flag, or `None` if it has no such flag.
///
/// The flag is `!` and `important` (any ASCII case) as the last tokens, with optional
/// whitespace and comments around and between them.
pub(crate) fn strip_important(value: &str) -> Option<&str> {
    // Common spelling, with the flag at the top level
    if let Some(rest) = value.trim_ascii_end().strip_suffix("!important") {
        if is_closed_plain(rest) {
            return Some(rest.trim_ascii_end());
        }
    }
    // Most values have no `!`, skip tokenizing them
    if !value.as_bytes().contains(&b'!') {
        return None;
    }
    let mut parser = cssparser::Parser::new(value);
    loop {
        if let Token::Delim('!') = parser.next().ok()? {
            // `!` is one byte, so it starts right before the current position
            let bang = parser.position().byte_index().saturating_sub(1);
            let state = parser.state();
            if parser.expect_ident_matching("important").is_ok() && parser.is_exhausted() {
                return Some(value[..bang].trim_ascii_end());
            }
            parser.reset(&state);
        }
    }
}

/// Whether `value` has no strings, comments, escapes or brackets other than balanced
/// parentheses, so what follows it is at the top level.
fn is_closed_plain(value: &str) -> bool {
    let mut depth = 0usize;
    for byte in value.bytes() {
        match byte {
            b'"' | b'\'' | b'\\' | b'/' | b'[' | b']' | b'{' | b'}' => return false,
            b'(' => depth = depth.saturating_add(1),
            b')' => {
                let Some(outer) = depth.checked_sub(1) else {
                    return false;
                };
                depth = outer;
            }
            _ => {}
        }
    }
    depth == 0
}

fn exhaust<'i>(input: &mut cssparser::Parser<'i>) -> &'i str {
    let start = input.position();
    while input.next().is_ok() {}
    input.slice_from(start)
}

/// Parser for qualified rules - a prelude + a simple {} block.
///
/// Usually these rules are a selector + list of declarations: `p { color: blue; font-size: 2px }`
impl<'i> cssparser::QualifiedRuleParser<'i> for RuleParser<'_, 'i> {
    type Prelude = &'i str;
    type QualifiedRule = ();
    type Error = ();

    fn parse_prelude(
        &mut self,
        input: &mut cssparser::Parser<'i>,
    ) -> Result<Self::Prelude, cssparser::ParseError<Self::Error>> {
        // Proceed with parsing until the end of the prelude.
        Ok(exhaust(input))
    }

    fn parse_block(
        &mut self,
        prelude: Self::Prelude,
        _: &ParserState,
        input: &mut cssparser::Parser<'i>,
    ) -> Result<Self::QualifiedRule, cssparser::ParseError<Self::Error>> {
        if self.skipped > 0 {
            exhaust(input);
            return Ok(());
        }
        let declarations = parse_declarations_into(input, self.declarations);
        self.rules.push(Rule {
            selectors: prelude,
            declarations,
            layer: self.layer,
        });
        Ok(())
    }
}

/// Prelude of an `@layer` rule: its raw text and the layer names in it.
pub(crate) struct LayerPrelude<'i> {
    raw: &'i str,
    names: Vec<&'i str>,
}

pub(crate) enum AtRulePrelude<'i> {
    Layer(LayerPrelude<'i>),
    /// A conditional group rule, whose block is a rule list.
    Conditional,
}

/// `<layer-name> = <ident> [ '.' <ident> ]*`, without whitespace.
fn parse_layer_name<'i>(
    input: &mut cssparser::Parser<'i>,
) -> Result<&'i str, cssparser::ParseError<()>> {
    input.skip_whitespace();
    let start = input.position();
    input.expect_ident()?;
    loop {
        let state = input.state();
        if !matches!(input.next_including_whitespace(), Ok(&Token::Delim('.'))) {
            input.reset(&state);
            break;
        }
        if !matches!(input.next_including_whitespace(), Ok(&Token::Ident(_))) {
            return Err(cssparser::ParseError::custom(()));
        }
    }
    let name = input.slice_from(start);
    input.expect_exhausted()?;
    Ok(name)
}

/// `@layer` rules are parsed. Conditional group rules are walked for the layers they declare
/// and then rejected like every other at-rule, as `style` attributes cannot hold them.
impl<'i> cssparser::AtRuleParser<'i> for RuleParser<'_, 'i> {
    type Prelude = AtRulePrelude<'i>;
    type AtRule = ();
    type Error = ();

    fn parse_prelude(
        &mut self,
        name: cssparser::CowRcStr<'i>,
        input: &mut cssparser::Parser<'i>,
    ) -> Result<Self::Prelude, cssparser::ParseError<Self::Error>> {
        if cssparser::match_ignore_ascii_case! { &name,
            "media" | "supports" | "container" | "document" | "-moz-document" => true,
            _ => false,
        } {
            exhaust(input);
            return Ok(AtRulePrelude::Conditional);
        }
        if !name.eq_ignore_ascii_case("layer") {
            return Err(cssparser::ParseError::custom(()));
        }
        let start = input.position();
        let names = if input.is_exhausted() {
            Vec::new()
        } else {
            input.parse_comma_separated(parse_layer_name)?
        };
        Ok(AtRulePrelude::Layer(LayerPrelude {
            raw: input.slice_from(start),
            names,
        }))
    }

    fn rule_without_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
    ) -> Result<Self::AtRule, ()> {
        let AtRulePrelude::Layer(prelude) = prelude else {
            return Err(());
        };
        if prelude.names.is_empty() {
            return Err(());
        }
        let layers = prelude
            .names
            .iter()
            .map(|name| self.layers.register(self.layer, name).0)
            .collect();
        self.record(Span::LayerStatement {
            start: start.position().byte_index(),
            prelude: prelude.raw,
            layers,
        });
        Ok(())
    }

    fn parse_block(
        &mut self,
        prelude: Self::Prelude,
        start: &ParserState,
        input: &mut cssparser::Parser<'i>,
    ) -> Result<Self::AtRule, cssparser::ParseError<Self::Error>> {
        let prelude = match prelude {
            AtRulePrelude::Layer(prelude) => prelude,
            AtRulePrelude::Conditional if !self.has_layers => {
                return Err(cssparser::ParseError::custom(()));
            }
            AtRulePrelude::Conditional => {
                self.skipped = self.skipped.saturating_add(1);
                collect_rules(&mut cssparser::StyleSheetParser::new(input, self));
                self.skipped = self.skipped.saturating_sub(1);
                // Rejected so the caller keeps the whole rule verbatim.
                return Err(cssparser::ParseError::custom(()));
            }
        };
        let (layer, created) = match prelude.names.as_slice() {
            [] => (self.layers.push(self.layer, None), true),
            [name] => self.layers.register(self.layer, name),
            _ => return Err(cssparser::ParseError::custom(())),
        };
        let outer = std::mem::replace(&mut self.layer, layer);
        collect_rules(&mut cssparser::StyleSheetParser::new(input, self));
        self.layer = outer;
        self.record(Span::LayerBlock {
            range: start.position().byte_index()..input.position().byte_index(),
            prelude: prelude.raw,
            layer,
            created,
        });
        Ok(())
    }
}

/// Parse a declaration within {} block: `color: blue`
impl<'i> cssparser::DeclarationParser<'i> for CSSDeclarationListParser {
    type Declaration = Declaration<'i>;
    type Error = ();

    fn parse_value(
        &mut self,
        name: Name<'i>,
        input: &mut cssparser::Parser<'i>,
        _declaration_start: &ParserState,
    ) -> Result<Self::Declaration, cssparser::ParseError<Self::Error>> {
        Ok((name, exhaust(input)))
    }
}

/// Parsing for at-rules, e.g: `@charset "utf-8";`
/// Since they are can not be inlined we use the default implementation, that rejects all at-rules.
impl<'i> cssparser::AtRuleParser<'i> for CSSDeclarationListParser {
    type Prelude = String;
    type AtRule = Declaration<'i>;
    type Error = ();
}

impl<'i> cssparser::RuleBodyItemParser<'i, Declaration<'i>, ()> for CSSDeclarationListParser {
    fn parse_declarations(&self) -> bool {
        true
    }

    fn parse_qualified(&self) -> bool {
        true
    }
}

impl<'i> cssparser::QualifiedRuleParser<'i> for CSSDeclarationListParser {
    type Prelude = String;
    type QualifiedRule = Declaration<'i>;
    type Error = ();
}

fn parse_declarations_into<'i>(
    input: &mut cssparser::Parser<'i>,
    declarations: &mut Vec<Declaration<'i>>,
) -> (usize, usize) {
    let mut parser = CSSDeclarationListParser;
    let parser = cssparser::RuleBodyParser::new(input, &mut parser);
    let start = declarations.len();
    for item in parser.flatten() {
        declarations.push(item);
    }
    (start, declarations.len())
}

#[cfg(test)]
mod tests {
    use super::split_selector_list;
    use test_case::test_case;

    #[test_case("h1, h2", &["h1", " h2"]; "plain list")]
    #[test_case("h1", &["h1"]; "single")]
    #[test_case("", &[""]; "empty")]
    #[test_case("h1,", &["h1", ""]; "trailing comma")]
    #[test_case(".x:not(.a, .b), p", &[".x:not(.a, .b)", " p"]; "inside parentheses")]
    #[test_case(":is(a, :not(b, c)), d", &[":is(a, :not(b, c))", " d"]; "nested parentheses")]
    #[test_case(r#"[title="a,b"], p"#, &[r#"[title="a,b"]"#, " p"]; "double-quoted string")]
    #[test_case("[title='a,b'], p", &["[title='a,b']", " p"]; "single-quoted string")]
    #[test_case(r#"[title="a'b,c"], p"#, &[r#"[title="a'b,c"]"#, " p"]; "other quote in string")]
    #[test_case(r#"[title="("], p"#, &[r#"[title="("]"#, " p"]; "parenthesis in string")]
    #[test_case(r#"[title="a\",b"], p"#, &[r#"[title="a\",b"]"#, " p"]; "escaped quote")]
    #[test_case(r".a\,b, p", &[r".a\,b", " p"]; "escaped comma")]
    #[test_case(r".a\(, p", &[r".a\(", " p"]; "escaped parenthesis")]
    #[test_case(".é\\é, p", &[".é\\é", " p"]; "escaped non-ASCII")]
    #[test_case("h1 /* a, b */, h2", &["h1 /* a, b */", " h2"]; "comment")]
    #[test_case("h1, /* a, b", &["h1", " /* a, b"]; "unterminated comment")]
    fn split(list: &str, expected: &[&str]) {
        assert_eq!(split_selector_list(list).collect::<Vec<_>>(), expected);
    }

    #[test_case("red !important", Some("red"); "plain")]
    #[test_case("red!important  ", Some("red"); "no spaces")]
    #[test_case("!important", Some(""); "flag only")]
    #[test_case("red !IMPORTANT", Some("red"); "uppercase")]
    #[test_case("red ! /* c */ Important /* d */", Some("red"); "comments")]
    #[test_case("rgb(0, 0, 0) !important", Some("rgb(0, 0, 0)"); "after function")]
    #[test_case(r#""a" !important"#, Some(r#""a""#); "after string")]
    #[test_case("red", None; "no flag")]
    #[test_case("important", None; "no bang")]
    #[test_case("red !important blue", None; "not last")]
    #[test_case(r"red \!important", None; "escaped bang")]
    #[test_case(r#""a !important""#, None; "inside string")]
    #[test_case("red /* !important */", None; "inside comment")]
    #[test_case("rgb(0 !important", None; "inside unclosed function")]
    #[test_case("var(--a, rgb(0 0 0)) !important", Some("var(--a, rgb(0 0 0))"); "after nested functions")]
    #[test_case("a) (b !important", None; "inside block after stray parenthesis")]
    #[test_case("[a !important", None; "inside unclosed bracket")]
    #[test_case("red !importantx", None; "longer ident")]
    fn strip_important(value: &str, expected: Option<&str>) {
        assert_eq!(super::strip_important(value), expected);
    }
}
