use super::{
    attributes::should_ignore,
    document::Document,
    node::{ElementData, Node, NodeData, NodeId},
};
use html5ever::{
    expanded_name, local_name, namespace_url, ns,
    tendril::{StrTendril, TendrilSink},
    tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink},
    Attribute, ExpandedName, QualName,
};
use std::borrow::Cow;

#[derive(Debug, Copy, Clone)]
pub(crate) enum InliningMode {
    /// Parse the input as a full HTML document.
    Document,
    /// Parse the input as an HTML fragment.
    Fragment,
}

/// Parse input bytes into an HTML document.
pub(crate) fn parse_with_options(
    bytes: &[u8],
    preallocate_node_capacity: usize,
    mode: InliningMode,
) -> Document {
    let sink = Sink {
        document: Document::with_capacity(preallocate_node_capacity),
    };
    let options = html5ever::ParseOpts::default();
    match mode {
        InliningMode::Document => html5ever::parse_document(sink, options)
            .from_utf8()
            .one(bytes),
        InliningMode::Fragment => {
            let mut document = html5ever::parse_fragment(
                sink,
                options,
                QualName::new(None, ns!(html), local_name!("")),
                vec![],
            )
            .from_utf8()
            .one(bytes);
            let document_id = NodeId::document_id();
            let context_element_id = NodeId::new(
                document_id
                    .get()
                    // The first one is a node representing the "" element passed above, then the
                    // second one is the "html" element.
                    .checked_add(2)
                    .expect("Document id is too small to overflow"),
            );
            document.reparent_children(context_element_id, document_id);
            document
        }
    }
}

/// Intermediary structure for parsing an HTML document.
/// It takes care of creating and appending nodes to the document as the parsing progresses.
struct Sink {
    /// An HTML document that is being parsed.
    document: Document,
}

impl Sink {
    /// Push a new node into the document.
    fn push_node(&mut self, data: NodeData) -> NodeId {
        self.document.push_node(data)
    }

    fn push_element(
        &mut self,
        name: QualName,
        attributes: Vec<Attribute>,
        inlining_ignored: bool,
    ) -> NodeId {
        let node_id = self.push_node(NodeData::Element {
            element: ElementData::new(name, attributes),
            inlining_ignored,
        });
        self.document.push_element_id(node_id);
        node_id
    }

    fn push_text(&mut self, text: StrTendril) -> NodeId {
        self.push_node(NodeData::Text { text })
    }

    fn push_comment(&mut self, text: StrTendril) -> NodeId {
        self.push_node(NodeData::Comment { text })
    }

    fn push_processing_instruction(&mut self, target: StrTendril, data: StrTendril) -> NodeId {
        self.push_node(NodeData::ProcessingInstruction { target, data })
    }

    fn push_doctype(&mut self, name: StrTendril) -> NodeId {
        self.push_node(NodeData::Doctype { name })
    }

    /// Append a new node or text to the document.
    fn append_impl<P, A>(&mut self, child: NodeOrText<NodeId>, previous: P, append: A)
    where
        P: FnOnce(&mut Document) -> Option<NodeId>,
        A: FnOnce(&mut Document, NodeId),
    {
        let new_node = match child {
            NodeOrText::AppendText(text) => {
                // If the previous node is a text node, append to it.
                // Otherwise create a new text node.
                if let Some(id) = previous(&mut self.document) {
                    if let Node {
                        data: NodeData::Text { text: existing },
                        ..
                    } = &mut self.document[id]
                    {
                        existing.push_tendril(&text);
                        return;
                    }
                }
                self.push_text(text)
            }
            NodeOrText::AppendNode(node) => node,
        };

        append(&mut self.document, new_node);
    }
}

impl TreeSink for Sink {
    type Handle = NodeId;
    type Output = Document;

    fn finish(self) -> Document {
        self.document
    }

    fn parse_error(&mut self, _msg: Cow<'static, str>) {}

    fn get_document(&mut self) -> NodeId {
        NodeId::document_id()
    }

    fn elem_name<'a>(&'a self, &target: &'a NodeId) -> ExpandedName<'a> {
        self.document[target]
            .as_element()
            // The `TreeSink` trait promises to never call this method on non-element node
            .expect("Not an element")
            .name
            .expanded()
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        _flags: ElementFlags,
    ) -> NodeId {
        // Determine if we should ignore inlining for this element based on its attributes
        let inlining_ignored = should_ignore(&attrs);

        // Determine if the element is a `style` element or a linked stylesheet (`link` with `rel="stylesheet"`).
        let (is_style, is_stylesheet) = {
            // If inlining is ignored, we consider neither to be true.
            if inlining_ignored {
                (false, false)
            } else if name.expanded() == expanded_name!(html "style") {
                (true, false)
            } else if name.expanded() == expanded_name!(html "link") {
                let mut rel_stylesheet = false;
                let mut href_non_empty = false;
                for attr in &attrs {
                    if attr.name.local == local_name!("rel") && attr.value == "stylesheet".into() {
                        rel_stylesheet = true;
                    }
                    // Skip links with empty `href` attributes
                    if attr.name.local == local_name!("href") && !attr.value.is_empty() {
                        href_non_empty = true;
                    }
                    if rel_stylesheet && href_non_empty {
                        break;
                    }
                }
                (false, rel_stylesheet && href_non_empty)
            } else {
                (false, false)
            }
        };
        let element = self.push_element(name, attrs, inlining_ignored);
        // Collect `style` tags and linked stylesheets separately to use them for CSS inlining later.
        if is_style {
            self.document.add_style(element);
        }
        if is_stylesheet {
            self.document.add_linked_stylesheet(element);
        }
        element
    }

    fn create_comment(&mut self, text: StrTendril) -> NodeId {
        self.push_comment(text)
    }

    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> NodeId {
        self.push_processing_instruction(target, data)
    }

    /// Append a node as the last child of the given node.
    fn append(&mut self, &parent: &NodeId, child: NodeOrText<NodeId>) {
        self.append_impl(
            child,
            |document| document[parent].last_child,
            |document, new_node| document.append(parent, new_node),
        );
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &NodeId,
        prev_element: &NodeId,
        child: NodeOrText<NodeId>,
    ) {
        if self.document[*element].parent.is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    /// Append a `DOCTYPE` element to the `Document` node.
    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
        let node = self.push_doctype(name);
        self.document.append(NodeId::document_id(), node);
    }

    fn get_template_contents(&mut self, &target: &NodeId) -> NodeId {
        target
    }

    /// Do two handles refer to the same node?
    fn same_node(&self, x: &NodeId, y: &NodeId) -> bool {
        x == y
    }

    fn set_quirks_mode(&mut self, _mode: QuirksMode) {}

    /// Append a node as the sibling immediately before the given node.
    fn append_before_sibling(&mut self, &sibling: &NodeId, child: NodeOrText<NodeId>) {
        self.append_impl(
            child,
            |document| document[sibling].previous_sibling,
            |document, node| document.insert_before(sibling, node),
        );
    }

    /// Add each attribute to the given element, if no attribute with that name already exists.
    fn add_attrs_if_missing(&mut self, &target: &NodeId, attrs: Vec<Attribute>) {
        let element = self.document[target]
            .as_element_mut()
            .expect("not an element");
        let attributes = &mut element.attributes;
        for attr in attrs {
            if attributes
                .attributes
                .iter()
                .any(|entry| entry.name == attr.name)
            {
                attributes.attributes.push(attr);
            }
        }
    }

    /// Detach the given node from its parent.
    fn remove_from_parent(&mut self, &target: &NodeId) {
        self.document.detach(target);
    }

    /// Remove all the children from node and append them to `new_parent`.
    fn reparent_children(&mut self, node: &NodeId, new_parent: &NodeId) {
        self.document.reparent_children(*node, *new_parent);
    }
}
