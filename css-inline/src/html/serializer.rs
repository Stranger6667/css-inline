use super::{
    attributes::Attributes,
    document::Document,
    node::{ElementData, NodeData, NodeId},
};
use html5ever::{local_name, namespace_url, ns, QualName};
use std::{io, io::Write};

pub(crate) fn serialize_to<W: Write>(
    document: &Document,
    writer: &mut W,
    keep_style_tags: bool,
) -> io::Result<()> {
    let sink = Sink::new(document, NodeId::document_id(), keep_style_tags);
    let mut serializer = HtmlSerializer::new(writer);
    sink.serialize(&mut serializer)
}

/// Intermediary structure for serializing an HTML document.
struct Sink<'a> {
    document: &'a Document,
    node: NodeId,
    keep_style_tags: bool,
}

impl<'a> Sink<'a> {
    fn new(document: &'a Document, node: NodeId, keep_style_tags: bool) -> Sink<'a> {
        Sink {
            document,
            node,
            keep_style_tags,
        }
    }
    #[inline]
    fn for_node(&self, node: NodeId) -> Sink<'a> {
        Sink::new(self.document, node, self.keep_style_tags)
    }
    #[inline]
    fn data(&self) -> &NodeData {
        &self.document[self.node].data
    }
    #[inline]
    fn should_skip_element(&self, element: &ElementData) -> bool {
        if element.name.local == local_name!("style") {
            !self.keep_style_tags
        } else {
            false
        }
    }
    fn serialize_children<W: Write>(&self, serializer: &mut HtmlSerializer<W>) -> io::Result<()> {
        for child in self.document.children(self.node) {
            self.for_node(child).serialize(serializer)?
        }
        Ok(())
    }
    fn serialize<W: Write>(&self, serializer: &mut HtmlSerializer<W>) -> io::Result<()> {
        match self.data() {
            NodeData::Element { element, .. } => {
                if self.should_skip_element(element) {
                    return Ok(());
                }
                serializer.start_elem(&element.name, &element.attributes)?;

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
    ignore_children: bool,
}

struct HtmlSerializer<Wr: Write> {
    writer: Wr,
    stack: Vec<ElemInfo>,
}

impl<Wr: Write> HtmlSerializer<Wr> {
    fn new(writer: Wr) -> Self {
        HtmlSerializer {
            writer,
            stack: vec![ElemInfo {
                ignore_children: false,
            }],
        }
    }

    fn parent(&mut self) -> &mut ElemInfo {
        self.stack.last_mut().expect("Stack is empty")
    }

    fn start_elem(&mut self, name: &QualName, attrs: &Attributes) -> io::Result<()> {
        if self.parent().ignore_children {
            self.stack.push(ElemInfo {
                ignore_children: true,
            });
            return Ok(());
        }

        self.writer.write_all(b"<")?;
        self.writer.write_all(name.local.as_bytes())?;
        for (name, value) in &attrs.map {
            self.writer.write_all(b" ")?;

            match name.ns {
                ns!() => (),
                ns!(xml) => self.writer.write_all(b"xml:")?,
                ns!(xmlns) => {
                    if name.local != local_name!("xmlns") {
                        self.writer.write_all(b"xmlns:")?;
                    }
                }
                ns!(xlink) => self.writer.write_all(b"xlink:")?,
                _ => {
                    self.writer.write_all(b"unknown_namespace:")?;
                }
            }

            self.writer.write_all(name.local.as_bytes())?;
            self.writer.write_all(b"=\"")?;
            self.writer.write_all(value.as_bytes())?;
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

        self.stack.push(ElemInfo { ignore_children });

        Ok(())
    }

    fn end_elem(&mut self, name: &QualName) -> io::Result<()> {
        let info = match self.stack.pop() {
            Some(info) => info,
            _ => panic!("no ElemInfo"),
        };
        if info.ignore_children {
            return Ok(());
        }

        self.writer.write_all(b"</")?;
        self.writer.write_all(name.local.as_bytes())?;
        self.writer.write_all(b">")
    }

    fn write_text(&mut self, text: &str) -> io::Result<()> {
        self.writer.write_all(text.as_bytes())
    }

    fn write_comment(&mut self, text: &str) -> io::Result<()> {
        self.writer.write_all(b"<!--")?;
        self.writer.write_all(text.as_bytes())?;
        self.writer.write_all(b"-->")
    }

    fn write_doctype(&mut self, name: &str) -> io::Result<()> {
        self.writer.write_all(b"<!DOCTYPE ")?;
        self.writer.write_all(name.as_bytes())?;
        self.writer.write_all(b">")
    }

    fn write_processing_instruction(&mut self, target: &str, data: &str) -> io::Result<()> {
        self.writer.write_all(b"<?")?;
        self.writer.write_all(target.as_bytes())?;
        self.writer.write_all(b" ")?;
        self.writer.write_all(data.as_bytes())?;
        self.writer.write_all(b">")
    }
}

#[cfg(test)]
mod tests {
    use super::Document;

    #[test]
    fn test_serialize() {
        let doc = Document::parse_with_options(b"<html><head><title>Test</title><style>h1 { color:blue; }</style><style>h1 { color:red; }</style></head>", 0);
        let mut buffer = Vec::new();
        doc.serialize(&mut buffer, true).expect("Should not fail");
        assert_eq!(buffer, b"<html><head><title>Test</title><style>h1 { color:blue; }</style><style>h1 { color:red; }</style></head><body></body></html>")
    }

    #[test]
    fn test_skip_style_tags() {
        let doc = Document::parse_with_options(b"<html><head><title>Test</title><style>h1 { color:blue; }</style><style>h1 { color:red; }</style></head>", 0);
        let mut buffer = Vec::new();
        doc.serialize(&mut buffer, false).expect("Should not fail");
        assert_eq!(
            buffer,
            b"<html><head><title>Test</title></head><body></body></html>"
        )
    }
}
