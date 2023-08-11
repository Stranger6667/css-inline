use super::attributes::Attributes;
use html5ever::{tendril::StrTendril, QualName};
use std::num::NonZeroUsize;

/// Single node in the DOM.
#[derive(Debug)]
pub(crate) struct Node {
    pub(crate) parent: Option<NodeId>,
    pub(crate) next_sibling: Option<NodeId>,
    pub(crate) previous_sibling: Option<NodeId>,
    pub(crate) first_child: Option<NodeId>,
    pub(crate) last_child: Option<NodeId>,
    /// Data specific to the type of node.
    pub(crate) data: NodeData,
}

impl Node {
    #[inline]
    pub(crate) fn new(data: NodeData) -> Node {
        Node {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            data,
        }
    }
    #[inline]
    pub(crate) fn as_element(&self) -> Option<&ElementData> {
        match &self.data {
            NodeData::Element { element: data, .. } => Some(data),
            _ => None,
        }
    }
    #[inline]
    pub(crate) fn as_element_mut(&mut self) -> Option<&mut ElementData> {
        match &mut self.data {
            NodeData::Element { element: data, .. } => Some(data),
            _ => None,
        }
    }
    #[inline]
    pub(crate) fn as_text(&self) -> Option<&str> {
        match &self.data {
            NodeData::Text { text } => Some(&**text),
            _ => None,
        }
    }
}

/// `NodeId` is a unique identifier for each `Node` in the document.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct NodeId(NonZeroUsize);

impl NodeId {
    #[inline]
    pub(super) fn new(value: usize) -> NodeId {
        NodeId(NonZeroUsize::new(value).expect("Value is zero"))
    }
    #[inline]
    pub(super) fn document_id() -> NodeId {
        NodeId::new(1)
    }
    #[inline]
    pub(super) fn get(self) -> usize {
        self.0.get()
    }
}

/// Data associated with a `Node`.
#[derive(Debug)]
pub(crate) enum NodeData {
    Document,
    Doctype {
        name: StrTendril,
    },
    Text {
        text: StrTendril,
    },
    Comment {
        text: StrTendril,
    },
    Element {
        element: ElementData,
        inlining_ignored: bool,
    },
    ProcessingInstruction {
        target: StrTendril,
        data: StrTendril,
    },
}

#[derive(Debug)]
pub(crate) struct ElementData {
    /// The name (tag) of the element.
    pub(crate) name: QualName,
    /// The attributes associated with the element.
    pub(crate) attributes: Attributes,
}

impl ElementData {
    #[inline]
    pub(crate) fn new(name: QualName, attributes: Vec<html5ever::Attribute>) -> ElementData {
        ElementData {
            name,
            attributes: Attributes::new(attributes),
        }
    }
}
