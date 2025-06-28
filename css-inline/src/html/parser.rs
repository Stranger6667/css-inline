use super::{
    attributes::should_ignore,
    document::Document,
    node::{ElementData, Node, NodeData, NodeId},
};
use html5ever::{
    expanded_name, local_name, ns,
    tendril::{StrTendril, TendrilSink},
    tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink},
    Attribute, QualName,
};
use std::{
    borrow::Cow,
    cell::{Ref, RefCell},
};

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
        document: RefCell::new(Document::with_capacity(preallocate_node_capacity)),
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
                false,
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
    document: RefCell<Document>,
}

impl Sink {
    /// Push a new node into the document.
    fn push_node(&self, data: NodeData) -> NodeId {
        self.document.borrow_mut().push_node(data)
    }

    fn push_element(
        &self,
        name: QualName,
        attributes: Vec<Attribute>,
        inlining_ignored: bool,
    ) -> NodeId {
        let node_id = self.push_node(NodeData::Element {
            element: ElementData::new(name, attributes),
            inlining_ignored,
        });
        self.document.borrow_mut().push_element_id(node_id);
        node_id
    }

    fn push_text(&self, text: StrTendril) -> NodeId {
        self.push_node(NodeData::Text { text })
    }

    fn push_comment(&self, text: StrTendril) -> NodeId {
        self.push_node(NodeData::Comment { text })
    }

    fn push_processing_instruction(&self, target: StrTendril, data: StrTendril) -> NodeId {
        self.push_node(NodeData::ProcessingInstruction { target, data })
    }

    fn push_doctype(&self, name: StrTendril) -> NodeId {
        self.push_node(NodeData::Doctype { name })
    }

    /// Append a new node or text to the document.
    fn append_impl<P, A>(&self, child: NodeOrText<NodeId>, previous: P, append: A)
    where
        P: FnOnce(&mut Document) -> Option<NodeId>,
        A: FnOnce(&mut Document, NodeId),
    {
        let new_node = match child {
            NodeOrText::AppendText(text) => {
                // If the previous node is a text node, append to it.
                // Otherwise create a new text node.
                let Some(id) = previous(&mut self.document.borrow_mut()) else {
                    let new_node = self.push_text(text);
                    append(&mut self.document.borrow_mut(), new_node);
                    return;
                };
                if let Node {
                    data: NodeData::Text { text: existing },
                    ..
                } = &mut self.document.borrow_mut()[id]
                {
                    existing.push_tendril(&text);
                    return;
                }
                self.push_text(text)
            }
            NodeOrText::AppendNode(node) => node,
        };

        append(&mut self.document.borrow_mut(), new_node);
    }
}

impl TreeSink for Sink {
    type Handle = NodeId;
    type Output = Document;
    type ElemName<'a>
        = Ref<'a, QualName>
    where
        Self: 'a;

    fn finish(self) -> Document {
        self.document.into_inner()
    }

    fn parse_error(&self, _msg: Cow<'static, str>) {}

    fn get_document(&self) -> NodeId {
        NodeId::document_id()
    }

    fn elem_name<'a>(&'a self, target: &NodeId) -> Self::ElemName<'a> {
        let document = self.document.borrow();
        let node = Ref::map(document, |document| &document[*target]);
        let element = Ref::map(node, |node| node.as_element().expect("Not an element"));
        Ref::map(element, |element| &element.name)
    }

    fn create_element(
        &self,
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
            self.document.borrow_mut().add_style(element);
        }
        if is_stylesheet {
            self.document.borrow_mut().add_linked_stylesheet(element);
        }
        element
    }

    fn create_comment(&self, text: StrTendril) -> NodeId {
        self.push_comment(text)
    }

    fn create_pi(&self, target: StrTendril, data: StrTendril) -> NodeId {
        self.push_processing_instruction(target, data)
    }

    /// Append a node as the last child of the given node.
    fn append(&self, &parent: &NodeId, child: NodeOrText<NodeId>) {
        self.append_impl(
            child,
            |document| document[parent].last_child,
            |document, new_node| document.append(parent, new_node),
        );
    }

    fn append_based_on_parent_node(
        &self,
        element: &NodeId,
        prev_element: &NodeId,
        child: NodeOrText<NodeId>,
    ) {
        if self.document.borrow()[*element].parent.is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    /// Append a `DOCTYPE` element to the `Document` node.
    fn append_doctype_to_document(
        &self,
        name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
        let node = self.push_doctype(name);
        self.document
            .borrow_mut()
            .append(NodeId::document_id(), node);
    }

    fn get_template_contents(&self, &target: &NodeId) -> NodeId {
        target
    }

    /// Do two handles refer to the same node?
    fn same_node(&self, x: &NodeId, y: &NodeId) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, _mode: QuirksMode) {}

    /// Append a node as the sibling immediately before the given node.
    fn append_before_sibling(&self, &sibling: &NodeId, child: NodeOrText<NodeId>) {
        self.append_impl(
            child,
            |document| document[sibling].previous_sibling,
            |document, node| document.insert_before(sibling, node),
        );
    }

    /// Add each attribute to the given element, if no attribute with that name already exists.
    fn add_attrs_if_missing(&self, &target: &NodeId, attrs: Vec<Attribute>) {
        let mut document = self.document.borrow_mut();
        let element = document[target].as_element_mut().expect("not an element");
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
    fn remove_from_parent(&self, &target: &NodeId) {
        self.document.borrow_mut().detach(target);
    }

    /// Remove all the children from node and append them to `new_parent`.
    fn reparent_children(&self, node: &NodeId, new_parent: &NodeId) {
        self.document
            .borrow_mut()
            .reparent_children(*node, *new_parent);
    }
}
