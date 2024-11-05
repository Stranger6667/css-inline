use super::{
    attributes::Attributes,
    document::Document,
    node::{ElementData, NodeData, NodeId},
    DocumentStyleMap, InliningMode,
};
use crate::{html::ElementStyleMap, parser, InlineError};
use html5ever::{local_name, namespace_url, ns, tendril::StrTendril, LocalName, QualName};
use smallvec::{smallvec, SmallVec};
use std::io::Write;

pub(crate) fn serialize_to<W: Write>(
    document: &Document,
    writer: &mut W,
    styles: DocumentStyleMap<'_>,
    keep_style_tags: bool,
    keep_link_tags: bool,
    mode: InliningMode,
) -> Result<(), InlineError> {
    let sink = Sink::new(
        document,
        NodeId::document_id(),
        keep_style_tags,
        keep_link_tags,
        mode,
    );
    let mut ser = HtmlSerializer::new(writer, styles);
    sink.serialize(&mut ser)
}

/// Intermediary structure for serializing an HTML document.
struct Sink<'a> {
    document: &'a Document,
    node: NodeId,
    keep_style_tags: bool,
    keep_link_tags: bool,
    inlining_mode: InliningMode,
}

impl<'a> Sink<'a> {
    fn new(
        document: &'a Document,
        node: NodeId,
        keep_style_tags: bool,
        keep_link_tags: bool,
        inlining_mode: InliningMode,
    ) -> Sink<'a> {
        Sink {
            document,
            node,
            keep_style_tags,
            keep_link_tags,
            inlining_mode,
        }
    }
    #[inline]
    fn for_node(&self, node: NodeId) -> Sink<'a> {
        Sink::new(
            self.document,
            node,
            self.keep_style_tags,
            self.keep_link_tags,
            self.inlining_mode,
        )
    }
    #[inline]
    fn data(&self) -> &NodeData {
        &self.document[self.node].data
    }
    #[inline]
    fn should_skip_element(&self, element: &ElementData) -> bool {
        if element.name.local == local_name!("style") {
            !self.keep_style_tags
                && element.attributes.get("data-css-inline".into()) != Some("keep")
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

    fn serialize_children<W: Write>(
        &self,
        serializer: &mut HtmlSerializer<'_, W>,
    ) -> Result<(), InlineError> {
        for child in self.document.children(self.node) {
            self.for_node(child).serialize(serializer)?;
        }
        Ok(())
    }

    fn serialize<W: Write>(
        &self,
        serializer: &mut HtmlSerializer<'_, W>,
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

                serializer.start_elem(&element.name, &element.attributes, style_node_id)?;

                self.serialize_children(serializer)?;

                serializer.end_elem(&element.name)?;
                Ok(())
            }
            NodeData::Document => self.serialize_children(serializer),
            NodeData::Doctype { name } => serializer.write_doctype(name),
            NodeData::Text { text: content } => serializer.write_text(content),
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
struct HtmlSerializer<'a, Wr: Write> {
    writer: Wr,
    styles: DocumentStyleMap<'a>,
    stack: Vec<ElemInfo>,
    style_buffer: SmallVec<[Vec<u8>; 8]>,
}

impl<'a, W: Write> HtmlSerializer<'a, W> {
    fn new(writer: W, styles: DocumentStyleMap<'a>) -> Self {
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
        }
    }

    fn parent(&mut self) -> &mut ElemInfo {
        self.stack.last_mut().expect("no parent ElemInfo")
    }

    fn write_escaped(&mut self, text: &str) -> Result<(), InlineError> {
        let mut last_end = 0;
        for (start, part) in text.match_indices(['&', '\u{00A0}', '<', '>']) {
            self.writer.write_all(
                text.get(last_end..start)
                    .expect("Invalid substring")
                    .as_bytes(),
            )?;
            match part {
                "&" => self.writer.write_all(b"&amp;")?,
                "\u{00A0}" => self.writer.write_all(b"&nbsp;")?,
                "<" => self.writer.write_all(b"&lt;")?,
                ">" => self.writer.write_all(b"&gt;")?,
                _ => unreachable!("Only the variants above are searched"),
            };
            last_end = start.checked_add(part.len()).expect("Size overflow");
        }
        self.writer.write_all(
            text.get(last_end..text.len())
                .expect("Invalid substring")
                .as_bytes(),
        )?;
        Ok(())
    }

    fn write_attributes(&mut self, text: &str) -> Result<(), InlineError> {
        let mut last_end = 0;
        for (start, part) in text.match_indices(['&', '\u{00A0}', '"']) {
            self.writer.write_all(
                text.get(last_end..start)
                    .expect("Invalid substring")
                    .as_bytes(),
            )?;
            match part {
                "&" => self.writer.write_all(b"&amp;")?,
                "\u{00A0}" => self.writer.write_all(b"&nbsp;")?,
                "\"" => self.writer.write_all(b"&quot;")?,
                _ => unreachable!("Only the variants above are searched"),
            };
            last_end = start.checked_add(part.len()).expect("Size overflow");
        }
        self.writer.write_all(
            text.get(last_end..text.len())
                .expect("Invalid substring")
                .as_bytes(),
        )?;
        Ok(())
    }

    fn start_elem(
        &mut self,
        name: &QualName,
        attrs: &Attributes,
        style_node_id: Option<NodeId>,
    ) -> Result<(), InlineError> {
        let html_name = match name.ns {
            ns!(html) => Some(name.local.clone()),
            _ => None,
        };

        if self.parent().ignore_children {
            self.stack.push(ElemInfo {
                html_name,
                ignore_children: true,
            });
            return Ok(());
        }

        let mut styles = if let Some(node_id) = style_node_id {
            self.styles.swap_remove(&node_id).map(|mut styles| {
                styles.sort_unstable_by(|_, (a, _), _, (b, _)| a.cmp(b));
                styles
            })
        } else {
            None
        };

        self.writer.write_all(b"<")?;
        self.writer.write_all(name.local.as_bytes())?;
        if let Some(class) = &attrs.class {
            self.writer.write_all(b" class=\"")?;
            self.writer.write_all(class.value.as_bytes())?;
            self.writer.write_all(b"\"")?;
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
            if attr.name.local.as_bytes() == b"style" {
                if let Some(new_styles) = &styles {
                    merge_styles(
                        &mut self.writer,
                        &attr.value,
                        new_styles,
                        &mut self.style_buffer,
                    )?;
                    styles = None;
                } else {
                    self.write_attributes(&attr.value)?;
                }
            } else {
                self.write_attributes(&attr.value)?;
            }
            self.writer.write_all(b"\"")?;
        }
        if let Some(styles) = &styles {
            self.writer.write_all(b" style=\"")?;
            for (property, (_, value)) in styles {
                write_declaration(&mut self.writer, property, value)?;
                self.writer.write_all(b";")?;
            }
            self.writer.write_all(b"\"")?;
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

#[inline]
fn write_declaration<Wr: Write>(
    writer: &mut Wr,
    name: &str,
    value: &str,
) -> Result<(), InlineError> {
    writer.write_all(name.as_bytes())?;
    writer.write_all(STYLE_SEPARATOR)?;
    write_declaration_value(writer, value)
}

#[inline]
fn write_declaration_value<Wr: Write>(writer: &mut Wr, value: &str) -> Result<(), InlineError> {
    let value = value.trim();
    if value.as_bytes().contains(&b'"') {
        // Roughly based on `str::replace`
        let mut last_end = 0;
        for (start, part) in value.match_indices('"') {
            writer.write_all(
                value
                    .get(last_end..start)
                    .expect("Invalid substring")
                    .as_bytes(),
            )?;
            writer.write_all(b"'")?;
            last_end = start.checked_add(part.len()).expect("Size overflow");
        }
        writer.write_all(
            value
                .get(last_end..value.len())
                .expect("Invalid substring")
                .as_bytes(),
        )?;
    } else {
        writer.write_all(value.as_bytes())?;
    };
    Ok(())
}

macro_rules! push_or_update {
    ($style_buffer:expr, $length:expr, $name: expr, $value:expr) => {{
        if let Some(style) = $style_buffer.get_mut($length) {
            style.clear();
            write_declaration(style, &$name, $value)?;
        } else {
            let value = $value.trim();
            let mut style = Vec::with_capacity(
                $name
                    .len()
                    .saturating_add(STYLE_SEPARATOR.len())
                    .saturating_add(value.len()),
            );
            write_declaration(&mut style, &$name, $value)?;
            $style_buffer.push(style);
        };
        $length = $length.saturating_add(1);
    }};
}

/// Merge a new set of styles into an current one, considering the rules of CSS precedence.
///
/// The merge process maintains the order of specificity and respects the `!important` rule in CSS.
fn merge_styles<Wr: Write>(
    writer: &mut Wr,
    current_style: &StrTendril,
    new_styles: &ElementStyleMap<'_>,
    declarations_buffer: &mut SmallVec<[Vec<u8>; 8]>,
) -> Result<(), InlineError> {
    // This function is designed with a focus on reusing existing allocations where possible
    // We start by parsing the current declarations in the "style" attribute
    let mut parser_input = cssparser::ParserInput::new(current_style);
    let mut parser = cssparser::Parser::new(&mut parser_input);
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
            write_declaration(buffer, &property, value)?;
        } else {
            let mut buffer = Vec::with_capacity(estimated_declaration_size);
            write_declaration(&mut buffer, &property, value)?;
            declarations_buffer.push(buffer);
        };
    }
    // Keep the number of current declarations to write them last as they have the precedence
    let current_declarations_count = parsed_declarations_count;
    // Next, we iterate over the new styles and merge them into our existing set
    // New rules will not override old ones unless they are marked as `!important`
    for (property, (_, value)) in new_styles {
        match (
            value.strip_suffix("!important"),
            declarations_buffer
                .iter_mut()
                .take(parsed_declarations_count)
                .find(|style| {
                    style.starts_with(property.as_bytes())
                        && style.get(property.len()..=property.len().saturating_add(1))
                            == Some(STYLE_SEPARATOR)
                }),
        ) {
            // The new rule is `!important` and there's an existing rule with the same name
            // In this case, we override the existing rule with the new one
            (Some(value), Some(buffer)) => {
                // We keep the rule name and the colon-space suffix - '<rule>: `
                buffer.truncate(property.len().saturating_add(STYLE_SEPARATOR.len()));
                write_declaration_value(buffer, value)?;
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

    let mut first = true;
    for range in [
        // First, write the new rules
        current_declarations_count..parsed_declarations_count,
        // Then, write the current rules
        0..current_declarations_count,
    ] {
        for declaration in &declarations_buffer[range] {
            if first {
                first = false;
            } else {
                writer.write_all(b";")?;
            }
            writer.write_all(declaration)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::html::InliningMode;

    use super::Document;
    use indexmap::IndexMap;

    #[test]
    fn test_serialize() {
        let doc = Document::parse_with_options(
            b"<html><head><style>h1 { color:blue; }</style><style>h1 { color:red }</style></head>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            IndexMap::default(),
            true,
            false,
            InliningMode::Document,
        )
        .expect("Should not fail");
        assert_eq!(buffer, b"<html><head><style>h1 { color:blue; }</style><style>h1 { color:red }</style></head><body></body></html>");
    }

    #[test]
    fn test_skip_style_tags() {
        let doc = Document::parse_with_options(
            b"<html><head><style>h1 { color:blue; }</style><style>h1 { color:red }</style></head>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            IndexMap::default(),
            false,
            false,
            InliningMode::Document,
        )
        .expect("Should not fail");
        assert_eq!(buffer, b"<html><head></head><body></body></html>");
    }

    #[test]
    fn test_escaped() {
        let doc = Document::parse_with_options(
            b"<!DOCTYPE html><html><head><title>& < > \xC2\xA0</title></head><body></body></html>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            IndexMap::default(),
            false,
            false,
            InliningMode::Document,
        )
        .expect("Should not fail");
        assert_eq!(buffer, b"<!DOCTYPE html><html><head><title>&amp; &lt; &gt; &nbsp;</title></head><body></body></html>");
    }

    #[test]
    fn test_untouched_style() {
        let doc = Document::parse_with_options(
            b"<html><body><p style=\"color:blue;\"></p></body></html>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            IndexMap::default(),
            false,
            false,
            InliningMode::Document,
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
            b"<!DOCTYPE html><html><head></head><body data-foo='& \xC2\xA0 \"'></body></html>",
            0,
            InliningMode::Document,
        );
        let mut buffer = Vec::new();
        doc.serialize(
            &mut buffer,
            IndexMap::default(),
            false,
            false,
            InliningMode::Document,
        )
        .expect("Should not fail");
        assert_eq!(buffer, b"<!DOCTYPE html><html><head></head><body data-foo=\"&amp; &nbsp; &quot;\"></body></html>");
    }
}
