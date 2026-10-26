use super::{
    attributes::Attributes,
    document::Document,
    node::{ElementData, NodeData, NodeId},
    variables::{contains_var, CustomProperties, NoVariables, VariableScope},
    DocumentStyleMap, InliningMode,
};
use crate::{html::ElementStyleMap, parser, InlineError};
use html5ever::{local_name, ns, tendril::StrTendril, LocalName, QualName};
use memchr::{memchr, memchr3_iter, memchr_iter, memmem};
use smallvec::{smallvec, SmallVec};
use std::io::Write;

/// Check if an element supports width/height HTML attributes.
#[inline]
fn supports_dimension_attrs(name: &LocalName) -> bool {
    matches!(
        *name,
        local_name!("table") | local_name!("td") | local_name!("th") | local_name!("img")
    )
}

/// Check if an element is a table element (supports percentage values).
#[inline]
fn is_table_element(name: &LocalName) -> bool {
    matches!(
        *name,
        local_name!("table") | local_name!("td") | local_name!("th")
    )
}

/// Extracted dimension value for HTML attribute.
#[derive(Clone, Copy)]
enum DimensionValue<'a> {
    /// Numeric value without suffix (for px/unitless)
    Numeric(&'a str),
    /// Numeric value that needs % suffix when written
    Percent(&'a str),
    /// The "auto" keyword
    Auto,
}

impl DimensionValue<'_> {
    #[inline]
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), InlineError> {
        match self {
            DimensionValue::Numeric(n) => writer.write_all(n.as_bytes())?,
            DimensionValue::Percent(n) => {
                writer.write_all(n.as_bytes())?;
                writer.write_all(b"%")?;
            }
            DimensionValue::Auto => writer.write_all(b"auto")?,
        }
        Ok(())
    }
}

/// Extract dimension value for HTML attribute from CSS value.
#[inline]
#[allow(clippy::arithmetic_side_effects)]
fn extract_dimension_value(value: &str, allow_percent: bool) -> Option<DimensionValue<'_>> {
    let value = value.trim();

    if value.eq_ignore_ascii_case("auto") {
        return Some(DimensionValue::Auto);
    }

    // Find where the numeric part ends
    let bytes = value.as_bytes();
    let mut end = 0;
    let mut has_dot = false;

    // Handle optional leading sign
    if bytes.first() == Some(&b'-') || bytes.first() == Some(&b'+') {
        end = 1;
    }

    // Parse digits and optional decimal point
    while end < bytes.len() {
        match bytes[end] {
            b'0'..=b'9' => end += 1,
            b'.' if !has_dot => {
                has_dot = true;
                end += 1;
            }
            _ => break,
        }
    }

    // Must have at least one digit
    if end == 0 || (end == 1 && (bytes[0] == b'-' || bytes[0] == b'+')) {
        return None;
    }

    let numeric_part = &value[..end];
    // Trim whitespace between number and unit (e.g., "100 px") for lenient parsing
    let unit_part = value[end..].trim();
    // Strip `!important` suffix if present
    let unit_part = parser::strip_important(unit_part).unwrap_or(unit_part);

    match unit_part {
        // Pixel values - strip the 'px' suffix
        "" | "px" => Some(DimensionValue::Numeric(numeric_part)),
        // Percentage - only allowed for table elements
        "%" if allow_percent => Some(DimensionValue::Percent(numeric_part)),
        // All other units are not supported
        _ => None,
    }
}

/// Find a style property value from stylesheet rules (not pre-existing inline styles).
#[inline]
fn find_style_value<'a>(styles: &'a ElementStyleMap<'_>, property: &str) -> Option<&'a str> {
    styles
        .iter()
        .rev()
        .find(|(name, _, _)| *name == property)
        .map(|(_, _, value)| *value)
}

/// Find a property value in an element's inline `style` attribute (last declaration wins).
fn find_inline_style_value<'a>(style: &'a str, property: &str) -> Option<&'a str> {
    let mut css_parser = cssparser::Parser::new(style);
    let mut declaration_parser = parser::CSSDeclarationListParser;
    let declarations = cssparser::RuleBodyParser::new(&mut css_parser, &mut declaration_parser);
    let mut found = None;
    for (name, value) in declarations.flatten() {
        if name.eq_ignore_ascii_case(property) {
            found = Some(value);
        }
    }
    found
}

/// Pick the cascade-effective value between an inline `style` declaration and a stylesheet rule.
/// Precedence (high to low): inline `!important`, stylesheet `!important`, inline, stylesheet.
fn effective_dimension<'a>(
    inline: Option<&'a str>,
    stylesheet: Option<&'a str>,
) -> Option<&'a str> {
    let important = |v: &str| parser::strip_important(v).is_some();
    if inline.is_some_and(important) {
        inline
    } else if stylesheet.is_some_and(important) {
        stylesheet
    } else {
        inline.or(stylesheet)
    }
}

#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
pub(crate) fn serialize_to<W: Write>(
    document: &Document,
    writer: &mut W,
    styles: DocumentStyleMap<'_>,
    keep_style_tags: bool,
    keep_link_tags: bool,
    minify_css: bool,
    at_rules: Option<&String>,
    mode: InliningMode,
    apply_width_attributes: bool,
    apply_height_attributes: bool,
    resolve_css_variables: bool,
) -> Result<(), InlineError> {
    let sink = Sink::new(
        document,
        NodeId::document_id(),
        keep_style_tags,
        keep_link_tags,
        minify_css,
        at_rules,
        mode,
        apply_width_attributes,
        apply_height_attributes,
    );
    if resolve_css_variables {
        let mut ser = HtmlSerializer::new(writer, styles, CustomProperties::default());
        sink.serialize(&mut ser)
    } else {
        let mut ser = HtmlSerializer::new(writer, styles, NoVariables);
        sink.serialize(&mut ser)
    }
}

/// Intermediary structure for serializing an HTML document.
#[allow(clippy::struct_excessive_bools)]
struct Sink<'a> {
    document: &'a Document,
    node: NodeId,
    keep_style_tags: bool,
    keep_link_tags: bool,
    minify_css: bool,
    at_rules: Option<&'a String>,
    inlining_mode: InliningMode,
    apply_width_attributes: bool,
    apply_height_attributes: bool,
}

impl<'a> Sink<'a> {
    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn new(
        document: &'a Document,
        node: NodeId,
        keep_style_tags: bool,
        keep_link_tags: bool,
        minify_css: bool,
        at_rules: Option<&'a String>,
        inlining_mode: InliningMode,
        apply_width_attributes: bool,
        apply_height_attributes: bool,
    ) -> Sink<'a> {
        Sink {
            document,
            node,
            keep_style_tags,
            keep_link_tags,
            minify_css,
            at_rules,
            inlining_mode,
            apply_width_attributes,
            apply_height_attributes,
        }
    }
    #[inline]
    fn for_node(&self, node: NodeId) -> Sink<'a> {
        Sink::new(
            self.document,
            node,
            self.keep_style_tags,
            self.keep_link_tags,
            self.minify_css,
            self.at_rules,
            self.inlining_mode,
            self.apply_width_attributes,
            self.apply_height_attributes,
        )
    }
    #[inline]
    fn data(&self) -> &NodeData {
        &self.document[self.node].data
    }
    #[inline]
    fn should_skip_element(&self, element: &ElementData) -> bool {
        if element.name.local == local_name!("style") {
            !self.keep_style_tags && element.attributes.get_css_inline() != Some("keep")
        } else if element.name.local == local_name!("link")
            && element.attributes.get(local_name!("rel")) == Some("stylesheet")
        {
            !self.keep_link_tags
        } else if element.name.local == local_name!("html") {
            matches!(self.inlining_mode, InliningMode::Fragment)
        } else {
            false
        }
    }

    fn serialize_children<W: Write, V: VariableScope>(
        &self,
        serializer: &mut HtmlSerializer<'_, W, V>,
    ) -> Result<(), InlineError> {
        for child in self.document.children(self.node) {
            self.for_node(child).serialize(serializer)?;
        }
        Ok(())
    }

    fn serialize<W: Write, V: VariableScope>(
        &self,
        serializer: &mut HtmlSerializer<'_, W, V>,
    ) -> Result<(), InlineError> {
        match self.data() {
            NodeData::Element {
                element,
                inlining_ignored,
            } => {
                if self.should_skip_element(element) {
                    return Ok(());
                }

                let style_node_id = if *inlining_ignored {
                    None
                } else {
                    Some(self.node)
                };

                serializer.start_elem(
                    &element.name,
                    &element.attributes,
                    style_node_id,
                    self.minify_css,
                    self.apply_width_attributes,
                    self.apply_height_attributes,
                )?;

                if element.name.local == local_name!("head") {
                    if let Some(at_rules) = &self.at_rules {
                        if !at_rules.is_empty() {
                            serializer.write_at_rules_style(at_rules)?;
                        }
                    }
                }

                self.serialize_children(serializer)?;

                serializer.end_elem(&element.name)?;
                Ok(())
            }
            NodeData::Document => self.serialize_children(serializer),
            NodeData::Doctype { name } => serializer.write_doctype(name),
            NodeData::Text { text } => serializer.write_text(text),
            NodeData::Comment { text } => serializer.write_comment(text),
            NodeData::ProcessingInstruction { target, data } => {
                serializer.write_processing_instruction(target, data)
            }
        }
    }
}

struct ElemInfo {
    html_name: Option<LocalName>,
    ignore_children: bool,
}

/// Inspired by HTML serializer from `html5ever`
/// Source: <https://github.com/servo/html5ever/blob/98d3c0cd01471af997cd60849a38da45a9414dfd/html5ever/src/serialize/mod.rs#L77>
struct HtmlSerializer<'a, Wr: Write, V: VariableScope> {
    writer: Wr,
    styles: DocumentStyleMap<'a>,
    stack: Vec<ElemInfo>,
    style_buffer: SmallVec<[Vec<u8>; 8]>,
    variables: V,
}

impl<'a, W: Write, V: VariableScope> HtmlSerializer<'a, W, V> {
    fn new(writer: W, styles: DocumentStyleMap<'a>, variables: V) -> Self {
        let mut stack = Vec::with_capacity(8);
        stack.push(ElemInfo {
            html_name: None,
            ignore_children: false,
        });
        HtmlSerializer {
            writer,
            styles,
            stack,
            style_buffer: smallvec![],
            variables,
        }
    }

    fn parent(&mut self) -> &mut ElemInfo {
        self.stack.last_mut().expect("no parent ElemInfo")
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn write_escaped(&mut self, text: &str) -> Result<(), InlineError> {
        let bytes = text.as_bytes();
        let mut last_end = 0;
        // U+00A0 is the only escaped non-ASCII character, and `0xC2` starts its UTF-8 form.
        // Without that byte, a SIMD scan for the three ASCII characters is enough.
        if memchr(0xC2, bytes).is_none() {
            for idx in memchr3_iter(b'&', b'<', b'>', bytes) {
                let escaped: &[u8] = match bytes[idx] {
                    b'&' => b"&amp;",
                    b'<' => b"&lt;",
                    _ => b"&gt;",
                };
                self.writer.write_all(&bytes[last_end..idx])?;
                self.writer.write_all(escaped)?;
                last_end = idx + 1;
            }
        } else {
            let mut idx = 0;
            while idx < bytes.len() {
                let (escaped, width): (&[u8], usize) = match bytes[idx] {
                    b'&' => (b"&amp;", 1),
                    b'<' => (b"&lt;", 1),
                    b'>' => (b"&gt;", 1),
                    0xC2 if bytes.get(idx + 1) == Some(&0xA0) => (b"&nbsp;", 2),
                    _ => {
                        idx += 1;
                        continue;
                    }
                };
                self.writer.write_all(&bytes[last_end..idx])?;
                self.writer.write_all(escaped)?;
                idx += width;
                last_end = idx;
            }
        }
        self.writer.write_all(&bytes[last_end..])?;
        Ok(())
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn write_attributes(&mut self, text: &str) -> Result<(), InlineError> {
        let bytes = text.as_bytes();
        let mut last_end = 0;

        // Scan for '&' (0x26), '"' (0x22), and 0xC2 (first byte of \u{00A0})
        for idx in memchr3_iter(b'&', b'"', 0xC2, bytes) {
            match bytes[idx] {
                b'&' => {
                    self.writer.write_all(&bytes[last_end..idx])?;
                    self.writer.write_all(b"&amp;")?;
                    last_end = idx + 1;
                }
                b'"' => {
                    self.writer.write_all(&bytes[last_end..idx])?;
                    self.writer.write_all(b"&quot;")?;
                    last_end = idx + 1;
                }
                0xC2 if bytes.get(idx + 1) == Some(&0xA0) => {
                    self.writer.write_all(&bytes[last_end..idx])?;
                    self.writer.write_all(b"&nbsp;")?;
                    last_end = idx + 2; // Skip both bytes of \u{00A0}
                }
                _ => {} // False positive for 0xC2 not followed by 0xA0
            }
        }
        self.writer.write_all(&bytes[last_end..])?;
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn start_elem(
        &mut self,
        name: &QualName,
        attrs: &Attributes,
        style_node_id: Option<NodeId>,
        minify_css: bool,
        apply_width_attributes: bool,
        apply_height_attributes: bool,
    ) -> Result<(), InlineError> {
        let html_name = match name.ns {
            ns!(html) => Some(name.local.clone()),
            _ => None,
        };
        self.variables.enter();

        if self.parent().ignore_children {
            self.stack.push(ElemInfo {
                html_name,
                ignore_children: true,
            });
            return Ok(());
        }

        let mut styles = if let Some(node_id) = style_node_id {
            self.styles.get_mut(node_id.get()).and_then(|slot| {
                slot.take().map(|mut styles| {
                    // Sort by specificity for consistent output order
                    styles.sort_unstable_by_key(|a| a.1);
                    styles
                })
            })
        } else {
            None
        };

        self.writer.write_all(b"<")?;
        self.writer.write_all(name.local.as_bytes())?;
        if let Some(class) = &attrs.class {
            self.writer.write_all(b" class=\"")?;
            self.write_attributes(&class.value)?;
            self.writer.write_all(b"\"")?;
        }

        // Extract and write width/height HTML attributes before styles is consumed
        if let Some(ref html_name) = html_name {
            if supports_dimension_attrs(html_name) {
                let allow_percent = is_table_element(html_name);
                // Resolve the cascade-effective value across the inline `style` and stylesheet.
                let style_attr = attrs.get(local_name!("style"));
                if apply_width_attributes && !attrs.contains(local_name!("width")) {
                    if let Some(dim) = effective_dimension(
                        style_attr.and_then(|s| find_inline_style_value(s, "width")),
                        styles.as_ref().and_then(|s| find_style_value(s, "width")),
                    )
                    .and_then(|v| extract_dimension_value(v, allow_percent))
                    {
                        self.writer.write_all(b" width=\"")?;
                        dim.write_to(&mut self.writer)?;
                        self.writer.write_all(b"\"")?;
                    }
                }
                if apply_height_attributes && !attrs.contains(local_name!("height")) {
                    if let Some(dim) = effective_dimension(
                        style_attr.and_then(|s| find_inline_style_value(s, "height")),
                        styles.as_ref().and_then(|s| find_style_value(s, "height")),
                    )
                    .and_then(|v| extract_dimension_value(v, allow_percent))
                    {
                        self.writer.write_all(b" height=\"")?;
                        dim.write_to(&mut self.writer)?;
                        self.writer.write_all(b"\"")?;
                    }
                }
            }
        }

        for attr in &attrs.attributes {
            self.writer.write_all(b" ")?;

            match attr.name.ns {
                ns!() => (),
                ns!(xml) => self.writer.write_all(b"xml:")?,
                ns!(xmlns) => {
                    if attr.name.local != local_name!("xmlns") {
                        self.writer.write_all(b"xmlns:")?;
                    }
                }
                ns!(xlink) => self.writer.write_all(b"xlink:")?,
                _ => {
                    self.writer.write_all(b"unknown_namespace:")?;
                }
            }

            self.writer.write_all(attr.name.local.as_bytes())?;
            self.writer.write_all(b"=\"")?;
            if attr.name.local == local_name!("style") {
                if let Some(new_styles) = &styles {
                    merge_styles(
                        &mut self.writer,
                        &attr.value,
                        new_styles,
                        &mut self.style_buffer,
                        minify_css,
                        &mut self.variables,
                    )?;
                    styles = None;
                } else if V::ENABLED
                    && (contains_var(attr.value.as_bytes())
                        || memmem::find(attr.value.as_bytes(), b"--").is_some())
                {
                    if style_node_id.is_some() {
                        // No rule matched, but the attribute itself has variables to resolve.
                        merge_styles(
                            &mut self.writer,
                            &attr.value,
                            &ElementStyleMap::new(),
                            &mut self.style_buffer,
                            minify_css,
                            &mut self.variables,
                        )?;
                    } else {
                        // Inlining is ignored: the attribute stays as it is, but its
                        // definitions apply to the descendants.
                        self.write_attributes(&attr.value)?;
                        let mut css_parser = cssparser::Parser::new(&attr.value);
                        let mut declaration_parser = parser::CSSDeclarationListParser;
                        let declarations = cssparser::RuleBodyParser::new(
                            &mut css_parser,
                            &mut declaration_parser,
                        );
                        for (name, value) in declarations.flatten() {
                            self.variables.define(name.as_bytes(), value.as_bytes());
                        }
                        self.variables.finish_definitions();
                    }
                } else {
                    self.write_attributes(&attr.value)?;
                }
            } else {
                self.write_attributes(&attr.value)?;
            }
            self.writer.write_all(b"\"")?;
        }
        if let Some(styles) = styles {
            for (property, _, value) in &styles {
                self.variables.define(property.as_bytes(), value.as_bytes());
            }
            self.variables.finish_definitions();
            // The attribute opens with the first declaration that is written.
            let mut opened = false;
            for (property, _, value) in &styles {
                let prefix: &[u8] = if !opened {
                    b" style=\""
                } else if minify_css {
                    b";"
                } else {
                    b""
                };
                if self.variables.write_declaration(
                    &mut self.writer,
                    prefix,
                    property,
                    value,
                    minify_css,
                )? {
                    opened = true;
                    if !minify_css {
                        self.writer.write_all(b";")?;
                    }
                }
            }
            if opened {
                self.writer.write_all(b"\"")?;
            }
        }
        self.writer.write_all(b">")?;

        let ignore_children = name.ns == ns!(html)
            && matches!(
                name.local,
                local_name!("area")
                    | local_name!("base")
                    | local_name!("basefont")
                    | local_name!("bgsound")
                    | local_name!("br")
                    | local_name!("col")
                    | local_name!("embed")
                    | local_name!("frame")
                    | local_name!("hr")
                    | local_name!("img")
                    | local_name!("input")
                    | local_name!("keygen")
                    | local_name!("link")
                    | local_name!("meta")
                    | local_name!("param")
                    | local_name!("source")
                    | local_name!("track")
                    | local_name!("wbr")
            );

        self.stack.push(ElemInfo {
            html_name,
            ignore_children,
        });

        Ok(())
    }

    fn end_elem(&mut self, name: &QualName) -> Result<(), InlineError> {
        self.variables.exit();
        let Some(info) = self.stack.pop() else {
            panic!("no ElemInfo")
        };
        if info.ignore_children {
            return Ok(());
        }

        self.writer.write_all(b"</")?;
        self.writer.write_all(name.local.as_bytes())?;
        self.writer.write_all(b">")?;
        Ok(())
    }

    fn write_text(&mut self, text: &str) -> Result<(), InlineError> {
        let escape = !matches!(
            self.parent().html_name,
            Some(
                local_name!("style")
                    | local_name!("script")
                    | local_name!("xmp")
                    | local_name!("iframe")
                    | local_name!("noembed")
                    | local_name!("noframes")
                    | local_name!("plaintext")
                    | local_name!("noscript")
            ),
        );

        if escape {
            self.write_escaped(text)?;
        } else {
            self.writer.write_all(text.as_bytes())?;
        }
        Ok(())
    }

    fn write_at_rules_style(&mut self, at_rules: &str) -> Result<(), InlineError> {
        self.writer.write_all(b"<style>")?;
        self.writer.write_all(at_rules.as_bytes())?;
        self.writer.write_all(b"</style>")?;
        Ok(())
    }

    fn write_comment(&mut self, text: &str) -> Result<(), InlineError> {
        self.writer.write_all(b"<!--")?;
        self.writer.write_all(text.as_bytes())?;
        self.writer.write_all(b"-->")?;
        Ok(())
    }

    fn write_doctype(&mut self, name: &str) -> Result<(), InlineError> {
        self.writer.write_all(b"<!DOCTYPE ")?;
        self.writer.write_all(name.as_bytes())?;
        self.writer.write_all(b">")?;
        Ok(())
    }

    fn write_processing_instruction(
        &mut self,
        target: &str,
        data: &str,
    ) -> Result<(), InlineError> {
        self.writer.write_all(b"<?")?;
        self.writer.write_all(target.as_bytes())?;
        self.writer.write_all(b" ")?;
        self.writer.write_all(data.as_bytes())?;
        self.writer.write_all(b">")?;
        Ok(())
    }
}

const STYLE_SEPARATOR: &[u8] = b": ";
const STYLE_SEPARATOR_MIN: &[u8] = b":";

#[inline]
pub(super) fn write_declaration<Wr: Write>(
    writer: &mut Wr,
    name: &str,
    value: &str,
    minify_css: bool,
) -> Result<(), InlineError> {
    writer.write_all(name.as_bytes())?;
    if minify_css {
        writer.write_all(STYLE_SEPARATOR_MIN)?;
    } else {
        writer.write_all(STYLE_SEPARATOR)?;
    }
    write_declaration_value(writer, value)
}

#[inline]
fn write_declaration_value<Wr: Write>(writer: &mut Wr, value: &str) -> Result<(), InlineError> {
    write_value_bytes(writer, value.trim().as_bytes())
}

/// Write a declaration value, replacing double quotes with single ones.
#[inline]
#[allow(clippy::arithmetic_side_effects)]
pub(super) fn write_value_bytes<Wr: Write>(
    writer: &mut Wr,
    bytes: &[u8],
) -> Result<(), InlineError> {
    let mut last_end = 0;
    for idx in memchr_iter(b'"', bytes) {
        writer.write_all(&bytes[last_end..idx])?;
        writer.write_all(b"'")?;
        last_end = idx + 1;
    }
    writer.write_all(&bytes[last_end..])?;
    Ok(())
}

macro_rules! push_or_update {
    ($style_buffer:expr, $length:expr, $name: expr, $value:expr, $minify_css:expr) => {{
        if let Some(style) = $style_buffer.get_mut($length) {
            style.clear();
            write_declaration(style, &$name, $value, $minify_css)?;
        } else {
            let value = $value.trim();
            let mut style = Vec::with_capacity(
                $name
                    .len()
                    .saturating_add(STYLE_SEPARATOR.len())
                    .saturating_add(value.len()),
            );
            write_declaration(&mut style, &$name, $value, $minify_css)?;
            $style_buffer.push(style);
        };
        $length = $length.saturating_add(1);
    }};
}

/// Merge a new set of styles into an current one, considering the rules of CSS precedence.
///
/// The merge process maintains the order of specificity and respects the `!important` rule in CSS.
fn merge_styles<Wr: Write, V: VariableScope>(
    writer: &mut Wr,
    current_style: &StrTendril,
    new_styles: &ElementStyleMap<'_>,
    declarations_buffer: &mut SmallVec<[Vec<u8>; 8]>,
    minify_css: bool,
    variables: &mut V,
) -> Result<(), InlineError> {
    // This function is designed with a focus on reusing existing allocations where possible
    // We start by parsing the current declarations in the "style" attribute
    let mut parser = cssparser::Parser::new(current_style);
    let mut declaration_parser = parser::CSSDeclarationListParser;
    let current_declarations = cssparser::RuleBodyParser::new(&mut parser, &mut declaration_parser);
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
            write_declaration(buffer, &property, value, minify_css)?;
        } else {
            let mut buffer = Vec::with_capacity(estimated_declaration_size);
            write_declaration(&mut buffer, &property, value, minify_css)?;
            declarations_buffer.push(buffer);
        }
    }
    // Keep the number of current declarations to write them last as they have the precedence
    let current_declarations_count = parsed_declarations_count;
    // Next, we iterate over the new styles and merge them into our existing set
    // New rules will not override old ones unless they are marked as `!important`
    let sep = if minify_css {
        STYLE_SEPARATOR_MIN
    } else {
        STYLE_SEPARATOR
    };
    for (property, _, value) in new_styles {
        match (
            parser::strip_important(value),
            declarations_buffer
                .iter_mut()
                .take(parsed_declarations_count)
                .find(|style| {
                    style.starts_with(property.as_bytes())
                        && style.get(property.len()..property.len().saturating_add(sep.len()))
                            == Some(sep)
                }),
        ) {
            // The new rule is `!important` and there's an existing rule with the same name
            // Only override if the existing inline rule is NOT `!important`.
            // Per CSS spec: inline `!important` takes precedence over stylesheet `!important`
            (Some(value), Some(buffer)) => {
                // Written from `&str` values, so it is UTF-8
                let inline_important = std::str::from_utf8(buffer)
                    .ok()
                    .and_then(parser::strip_important)
                    .is_some();
                if !inline_important {
                    buffer.truncate(property.len().saturating_add(sep.len()));
                    write_declaration_value(buffer, value)?;
                    buffer.extend_from_slice(b" !important");
                }
            }
            // There's no existing rule with the same name, but the new rule is `!important`
            (Some(value), None) => {
                push_or_update!(
                    declarations_buffer,
                    parsed_declarations_count,
                    property,
                    value,
                    minify_css
                );
                // `push_or_update!` increments `parsed_declarations_count`, so subtract 1
                // to get the slot that was just written and append the `!important` suffix.
                if let Some(buf) =
                    declarations_buffer.get_mut(parsed_declarations_count.saturating_sub(1))
                {
                    buf.extend_from_slice(b" !important");
                }
            }
            // There's no existing rule with the same name, and the new rule is not `!important`
            // In this case, we just add the new rule as-is
            (None, None) => push_or_update!(
                declarations_buffer,
                parsed_declarations_count,
                property,
                value,
                minify_css
            ),
            // Rule exists and the new one is not `!important` - leave the existing rule as-is and
            // ignore the new one.
            (None, Some(_)) => {}
        }
    }

    for declaration in &declarations_buffer[..parsed_declarations_count] {
        variables.define_merged(declaration, sep.len());
    }
    variables.finish_definitions();
    let mut first = true;
    for range in [
        // First, write the new rules
        current_declarations_count..parsed_declarations_count,
        // Then, write the current rules
        0..current_declarations_count,
    ] {
        for declaration in &declarations_buffer[range] {
            let prefix: &[u8] = if first { b"" } else { b";" };
            if variables.write_merged(writer, prefix, declaration, sep.len())? {
                first = false;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::html::InliningMode;
    use test_case::test_case;

    use super::Document;

    fn serialize_body(body: &str) -> String {
        let html = format!("<html><head></head><body>{body}</body></html>");
        let doc = Document::parse_with_options(html.as_bytes(), 0, InliningMode::Document);
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            vec![None; doc.nodes.len()],
            true,
            false,
            false,
            None,
            InliningMode::Document,
            false,
            false,
            false,
        )
        .expect("Should not fail");
        let output = String::from_utf8(buffer).expect("Valid UTF-8");
        output
            .strip_prefix("<html><head></head><body>")
            .and_then(|rest| rest.strip_suffix("</body></html>"))
            .expect("Unexpected document shape")
            .to_string()
    }

    #[test_case("", ""; "empty")]
    #[test_case("plain text", "plain text"; "nothing to escape")]
    #[test_case("&amp;", "&amp;"; "ampersand")]
    #[test_case("&lt;", "&lt;"; "less than")]
    #[test_case("&gt;", "&gt;"; "greater than")]
    #[test_case("&nbsp;", "&nbsp;"; "non-breaking space")]
    #[test_case("\"'", "\"'"; "quotes untouched")]
    #[test_case("&amp;abc", "&amp;abc"; "escape at start")]
    #[test_case("abc&gt;", "abc&gt;"; "escape at end")]
    #[test_case("&amp;&lt;&gt;&nbsp;&nbsp;&amp;", "&amp;&lt;&gt;&nbsp;&nbsp;&amp;"; "consecutive")]
    #[test_case("é&amp;ü", "é&amp;ü"; "multibyte neighbours")]
    #[test_case("¡&nbsp;¡", "¡&nbsp;¡"; "other 0xC2 lead bytes")]
    #[test_case("à", "à"; "0xA0 continuation byte")]
    #[test_case("Âà", "Âà"; "0xC3 0x82 then 0xC3 0xA0")]
    #[test_case(
        "Lorem ipsum dolor sit amet, consectetur &amp; adipiscing elit &lt;sed&gt; do eiusmod tempor&nbsp;€",
        "Lorem ipsum dolor sit amet, consectetur &amp; adipiscing elit &lt;sed&gt; do eiusmod tempor&nbsp;€";
        "long mixed"
    )]
    fn test_escape_text(input: &str, expected: &str) {
        assert_eq!(
            serialize_body(&format!("<p>{input}</p>")),
            format!("<p>{expected}</p>")
        );
    }

    #[test_case("", ""; "empty")]
    #[test_case("plain", "plain"; "nothing to escape")]
    #[test_case("&amp;", "&amp;"; "ampersand")]
    #[test_case("&quot;", "&quot;"; "double quote")]
    #[test_case("&nbsp;", "&nbsp;"; "non-breaking space")]
    #[test_case("&lt;&gt;'", "<>'"; "angle brackets and single quote untouched")]
    #[test_case("&amp;abc", "&amp;abc"; "escape at start")]
    #[test_case("abc&quot;", "abc&quot;"; "escape at end")]
    #[test_case("&amp;&quot;&nbsp;&nbsp;&amp;", "&amp;&quot;&nbsp;&nbsp;&amp;"; "consecutive")]
    #[test_case("é&amp;ü", "é&amp;ü"; "multibyte neighbours")]
    #[test_case("¡&nbsp;¡", "¡&nbsp;¡"; "other 0xC2 lead bytes")]
    #[test_case("à", "à"; "0xA0 continuation byte")]
    fn test_escape_attribute(input: &str, expected: &str) {
        assert_eq!(
            serialize_body(&format!("<p data-foo=\"{input}\"></p>")),
            format!("<p data-foo=\"{expected}\"></p>")
        );
    }

    #[test_case("script"; "script")]
    #[test_case("style"; "style")]
    #[test_case("xmp"; "xmp")]
    fn test_raw_text_untouched(tag: &str) {
        let body = format!("<{tag}>a & b < c > d \" \u{00A0} é</{tag}>");
        assert_eq!(serialize_body(&body), body);
    }

    #[test]
    fn test_serialize() {
        let doc = Document::parse_with_options(
            "<html><head><style>h1 { color:blue; }</style><style>h1 { color:red }</style></head>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            vec![None; doc.nodes.len()],
            true,
            false,
            false,
            None,
            InliningMode::Document,
            false,
            false,
            false,
        )
        .expect("Should not fail");
        assert_eq!(buffer, b"<html><head><style>h1 { color:blue; }</style><style>h1 { color:red }</style></head><body></body></html>");
    }

    #[test]
    fn test_skip_style_tags() {
        let doc = Document::parse_with_options(
            "<html><head><style>h1 { color:blue; }</style><style>h1 { color:red }</style></head>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            vec![None; doc.nodes.len()],
            false,
            false,
            false,
            None,
            InliningMode::Document,
            false,
            false,
            false,
        )
        .expect("Should not fail");
        assert_eq!(buffer, b"<html><head></head><body></body></html>");
    }

    #[test]
    fn test_escaped() {
        let doc = Document::parse_with_options(
            "<!DOCTYPE html><html><head><title>& < > \u{a0}</title></head><body></body></html>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            vec![None; doc.nodes.len()],
            false,
            false,
            false,
            None,
            InliningMode::Document,
            false,
            false,
            false,
        )
        .expect("Should not fail");
        assert_eq!(buffer, b"<!DOCTYPE html><html><head><title>&amp; &lt; &gt; &nbsp;</title></head><body></body></html>");
    }

    #[test]
    fn test_untouched_style() {
        let doc = Document::parse_with_options(
            "<html><body><p style=\"color:blue;\"></p></body></html>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            vec![None; doc.nodes.len()],
            false,
            false,
            false,
            None,
            InliningMode::Document,
            false,
            false,
            false,
        )
        .expect("Should not fail");
        assert_eq!(
            buffer,
            b"<html><head></head><body><p style=\"color:blue;\"></p></body></html>"
        );
    }

    #[test]
    fn test_attributes() {
        let doc = Document::parse_with_options(
            "<!DOCTYPE html><html><head></head><body data-foo='& \u{a0} \"'></body></html>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            vec![None; doc.nodes.len()],
            false,
            false,
            false,
            None,
            InliningMode::Document,
            false,
            false,
            false,
        )
        .expect("Should not fail");
        assert_eq!(buffer, b"<!DOCTYPE html><html><head></head><body data-foo=\"&amp; &nbsp; &quot;\"></body></html>");
    }

    #[test]
    fn test_class_attribute_escaped() {
        let doc = Document::parse_with_options(
            "<!DOCTYPE html><html><head></head><body class='\"><img src=x onerror=alert(1)> &amp; \u{a0}'></body></html>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            vec![None; doc.nodes.len()],
            false,
            false,
            false,
            None,
            InliningMode::Document,
            false,
            false,
            false,
        )
        .expect("Should not fail");
        assert_eq!(buffer, b"<!DOCTYPE html><html><head></head><body class=\"&quot;><img src=x onerror=alert(1)> &amp; &nbsp;\"></body></html>");
    }

    #[test]
    fn test_keep_at_rules_tags() {
        let doc = Document::parse_with_options(
            "<html><head><style>h1 { color:red }</style></head>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            vec![None; doc.nodes.len()],
            false,
            false,
            false,
            Some(&String::from(
                "@media (max-width: 600px) { h1 { font-size: 18px; } }",
            )),
            InliningMode::Document,
            false,
            false,
            false,
        )
        .expect("Should not fail");
        assert_eq!(buffer, b"<html><head><style>@media (max-width: 600px) { h1 { font-size: 18px; } }</style></head><body></body></html>");
    }
}
