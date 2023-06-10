use super::{
    document::Document,
    node::{ElementData, NodeData, NodeId},
};
use html5ever::{
    local_name, serialize,
    serialize::{Serialize, SerializeOpts, Serializer, TraversalScope},
};
use std::{io, io::Write};

pub(crate) fn serialize_to<W: Write>(
    document: &Document,
    writer: &mut W,
    skip_style_tags: bool,
) -> io::Result<()> {
    let sink = Sink::new(document, NodeId::document_id(), skip_style_tags);
    serialize(writer, &sink, SerializeOpts::default())
}

/// Intermediary structure for serializing an HTML document.
struct Sink<'a> {
    document: &'a Document,
    node: NodeId,
    skip_style_tags: bool,
}

impl<'a> Sink<'a> {
    fn new(document: &'a Document, node: NodeId, skip_style_tags: bool) -> Sink<'a> {
        Sink {
            document,
            node,
            skip_style_tags,
        }
    }
    fn for_node(&self, node: NodeId) -> Sink<'a> {
        Sink::new(self.document, node, self.skip_style_tags)
    }
    fn data(&self) -> &NodeData {
        &self.document[self.node].data
    }
    fn should_skip_element(&self, element: &ElementData) -> bool {
        self.skip_style_tags && element.name.local == local_name!("style")
    }
    fn serialize_children<S: Serializer>(&self, serializer: &mut S) -> io::Result<()> {
        for child in self.document.children(self.node) {
            Serialize::serialize(
                &self.for_node(child),
                serializer,
                TraversalScope::IncludeNode,
            )?
        }
        Ok(())
    }
}

impl<'a> Serialize for Sink<'a> {
    fn serialize<S: Serializer>(&self, serializer: &mut S, _: TraversalScope) -> io::Result<()> {
        // TODO: Consider iteration over recursion
        match self.data() {
            NodeData::Element { element, .. } => {
                if self.should_skip_element(element) {
                    return Ok(());
                }
                serializer.start_elem(
                    element.name.clone(),
                    element
                        .attributes
                        .map
                        .iter()
                        .map(|(name, value)| (name, &**value)),
                )?;

                self.serialize_children(serializer)?;

                serializer.end_elem(element.name.clone())?;
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

#[cfg(test)]
mod tests {
    use super::Document;

    #[test]
    fn test_serialize() {
        let doc = Document::parse_with_options(b"<html><head><title>Test</title><style>h1 { color:blue; }</style><style>h1 { color:red; }</style></head>", 0);
        let mut buffer = Vec::new();
        doc.serialize(&mut buffer, false).expect("Should not fail");
        assert_eq!(buffer, b"<html><head><title>Test</title><style>h1 { color:blue; }</style><style>h1 { color:red; }</style></head><body></body></html>")
    }

    #[test]
    fn test_skip_style_tags() {
        let doc = Document::parse_with_options(b"<html><head><title>Test</title><style>h1 { color:blue; }</style><style>h1 { color:red; }</style></head>", 0);
        let mut buffer = Vec::new();
        doc.serialize(&mut buffer, true).expect("Should not fail");
        assert_eq!(
            buffer,
            b"<html><head><title>Test</title></head><body></body></html>"
        )
    }
}
