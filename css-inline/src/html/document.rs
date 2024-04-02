use super::{
    element::Element,
    iter::{select, Select},
    node::{Node, NodeData, NodeId},
    parser,
    selectors::ParseError,
    serializer::serialize_to,
    InliningMode,
};
use crate::{html::DocumentStyleMap, InlineError};
use html5ever::local_name;
use selectors::NthIndexCache;
use std::{fmt, fmt::Formatter, io::Write, iter::successors};

/// HTML document representation.
///
/// A `Document` holds a collection of nodes, with each node representing an HTML element.
/// Nodes are interconnected to form a tree-like structure, mimicking the HTML DOM tree.
/// The struct also keeps track of nodes that contain CSS styles or refer to CSS stylesheets.
///
/// Here is an example of how nodes within a `Document` could be arranged:
///
/// ```text
///                    Document
///                       ↓
///                      [N1]
///                     /   \
///                    /     \
///                   /       \
///                  /         \
///                 /           \
///               [N2]<->[N3]<->[N4]
///               /  \          /   \
///              /    \        /     \
///            [N5]<->[N6]   [N7]<->[N8]
/// ```
///
/// Each Node within the `Document` is interconnected with its siblings, and has a parent-child
/// relationship with its descendants.
pub(crate) struct Document {
    pub(crate) nodes: Vec<Node>,
    /// Ids of Element nodes & caches for their nth index selectors.
    pub(crate) elements: Vec<NodeId>,
    /// Ids of `style` nodes.
    styles: Vec<NodeId>,
    /// Ids of `link` nodes, specifically those with the `rel` attribute value set as `stylesheet`.
    /// They represent the locations (URLs) of all linked stylesheet resources in the document.
    linked_stylesheets: Vec<NodeId>,
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Document")
            .field("nodes", &self.nodes)
            .field("styles", &self.styles)
            .field("linked_stylesheets", &self.linked_stylesheets)
            .finish_non_exhaustive()?;
        Ok(())
    }
}

impl Document {
    pub(crate) fn parse_with_options(
        bytes: &[u8],
        preallocate_node_capacity: usize,
        mode: InliningMode,
    ) -> Document {
        parser::parse_with_options(bytes, preallocate_node_capacity, mode)
    }

    pub(super) fn with_capacity(capacity: usize) -> Self {
        // Dummy node at index 0 so that other indices fit in NonZero
        let mut nodes = vec![Node::new(NodeData::Document), Node::new(NodeData::Document)];
        // Usually there are a lot of nodes, hence, reserve some space for them
        nodes.reserve(capacity);
        Document {
            nodes,
            elements: Vec::with_capacity(capacity),
            styles: Vec::new(),
            linked_stylesheets: Vec::new(),
        }
    }

    #[inline]
    pub(super) fn as_element(&self, node_id: NodeId) -> Option<Element<'_>> {
        if let NodeData::Element { element, .. } = &self[node_id].data {
            Some(Element::new(self, node_id, element))
        } else {
            None
        }
    }

    /// Add a new `style` element node.
    pub(super) fn add_style(&mut self, node: NodeId) {
        self.styles.push(node);
    }

    /// Iterator over blocks of CSS defined inside `style` tags.
    pub(crate) fn styles(&self) -> impl Iterator<Item = &str> + '_ {
        self.styles.iter().filter_map(|node_id| {
            self[*node_id]
                .first_child
                .and_then(|child_id| self[child_id].as_text())
        })
    }

    /// Iterate over `href` attribute values of `link[rel~=stylesheet]` tags.
    pub(crate) fn stylesheets(&self) -> impl Iterator<Item = &str> + '_ {
        self.linked_stylesheets.iter().filter_map(|node_id| {
            self[*node_id]
                .as_element()
                .and_then(|data| data.attributes.get(local_name!("href")))
        })
    }

    /// Add a new linked stylesheet location.
    pub(super) fn add_linked_stylesheet(&mut self, node: NodeId) {
        self.linked_stylesheets.push(node);
    }

    /// Add a new node to the nodes vector, returning its index.
    pub(super) fn push_node(&mut self, node: NodeData) -> NodeId {
        let next_index = self.nodes.len();
        self.nodes.push(Node::new(node));
        NodeId::new(next_index)
    }

    #[inline]
    pub(super) fn push_element_id(&mut self, node: NodeId) {
        self.elements.push(node);
    }

    /// Detach a node from its siblings and its parent.
    ///
    /// Before:
    ///
    ///   [Parent]
    ///     ↓
    ///    ... [Previous] <--> [Node] <--> [Next] ...
    ///
    /// After:
    ///
    ///   [Parent]
    ///     ↓
    ///    ... [Previous] <--> [Next] ...
    pub(super) fn detach(&mut self, node: NodeId) {
        // Save references to the parent and sibling nodes of the node being detached.
        let (parent, previous_sibling, next_sibling) = {
            let node = &mut self[node];
            (
                node.parent.take(),
                node.previous_sibling.take(),
                node.next_sibling.take(),
            )
        };

        if let Some(next_sibling) = next_sibling {
            // Point next sibling one step back to bypass the detached node
            self[next_sibling].previous_sibling = previous_sibling;
        } else if let Some(parent) = parent {
            // No next sibling - this node was the last child of the parent node, now the previous
            // sibling becomes the last child
            self[parent].last_child = previous_sibling;
        }

        if let Some(previous_sibling) = previous_sibling {
            // Point the previous sibling one step forward to bypass the detached node
            self[previous_sibling].next_sibling = next_sibling;
        } else if let Some(parent) = parent {
            // No previous sibling - this node was the first child of the parent node, now the next
            // sibling becomes the first child
            self[parent].first_child = next_sibling;
        }
    }

    /// Remove all the children from node and append them to `new_parent`.
    pub(super) fn reparent_children(&mut self, node: NodeId, new_parent: NodeId) {
        let mut next_child = self[node].first_child;
        while let Some(child) = next_child {
            self.append(new_parent, child);
            next_child = self[child].next_sibling;
        }
    }

    /// Append a new child node to a parent node.
    ///
    /// If the parent node already has children. Before:
    ///
    ///   [Parent]
    ///      ↓
    ///   [Child1] <--> [Child2] <--> ...
    ///
    /// After:
    ///
    ///   [Parent]
    ///      ↓
    ///   [Child1] <--> [Child2] <--> [New] ...
    ///
    /// If the parent node has no children. Before:
    ///
    ///   [Parent]
    ///
    /// After:
    ///
    ///   [Parent]
    ///     ↓
    ///   [New]
    ///
    /// So, [New} becomes the first child of [Parent].
    pub(super) fn append(&mut self, parent: NodeId, node: NodeId) {
        // Detach `node` from its current parent (if any)
        self.detach(node);

        // Set `node` parent to the specified parent
        self[node].parent = Some(parent);

        if let Some(last_child) = self[parent].last_child.take() {
            // Connect `node` with the last child (if any) by adding `node` after it
            self[node].previous_sibling = Some(last_child);
            self[last_child].next_sibling = Some(node);
        } else {
            // No last child - it becomes the first child
            self[parent].first_child = Some(node);
        }

        // Now, `node` is the last child of the new parent
        self[parent].last_child = Some(node);
    }

    /// Insert a new node right before a given sibling node.
    ///
    /// In general, this method transforms this relationship:
    ///
    ///   ... <--> [Previous] <--> [Sibling] <--> [Next] --> ...
    ///
    /// into this:
    ///
    ///   ... <--> [Previous] <--> [New] <--> [Sibling] <--> [Next] <--> ...
    ///
    /// If [Sibling] node is the first child (i.e., no [Previous] exists), the method also updates the parent node:
    ///
    /// Before:
    ///
    ///   [Parent]
    ///     ↓
    ///   [Sibling] <--> [Next] <--> ...
    ///
    /// After:
    ///
    ///   [Parent]
    ///     ↓
    ///   [New] <--> [Sibling] <--> [Next] <--> ...
    ///
    /// So, [New] becomes the first child of [Parent].
    pub(super) fn insert_before(&mut self, sibling: NodeId, node: NodeId) {
        // Detach `node` from its current parent (if any)
        self.detach(node);

        // Set `node` parent to `sibling` parent
        self[node].parent = self[sibling].parent;

        // As it is inserted before, then `next_sibling` should point to `sibling`
        self[node].next_sibling = Some(sibling);

        if let Some(previous_sibling) = self[sibling].previous_sibling.take() {
            // Connect `node` with the previous sibling (if any)
            self[node].previous_sibling = Some(previous_sibling);
            self[previous_sibling].next_sibling = Some(node);
        } else if let Some(parent) = self[sibling].parent {
            // No previous sibling - then it is the first child of its parent, so the parent node
            // should be updated too
            self[parent].first_child = Some(node);
        }

        // Now `node` is the previous sibling of the `sibling` node
        self[sibling].previous_sibling = Some(node);
    }

    /// Returns an iterator over the direct children of a node.
    pub(super) fn children(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        successors(self[node].first_child, |&node| self[node].next_sibling)
    }

    /// Serialize the document to HTML string.
    pub(crate) fn serialize<W: Write>(
        &self,
        writer: &mut W,
        styles: DocumentStyleMap<'_>,
        keep_style_tags: bool,
        keep_link_tags: bool,
        mode: InliningMode,
    ) -> Result<(), InlineError> {
        serialize_to(self, writer, styles, keep_style_tags, keep_link_tags, mode)
    }

    /// Filter this node iterator to elements matching the given selectors.
    pub(crate) fn select<'a, 'b, 'c>(
        &'a self,
        selectors: &'b str,
        caches: &'c mut [NthIndexCache],
    ) -> Result<Select<'a, 'c>, ParseError<'b>> {
        select(self, selectors, caches)
    }

    pub(crate) fn build_caches(&self) -> Vec<NthIndexCache> {
        (0..self.elements.len())
            .map(|_| NthIndexCache::default())
            .collect()
    }
}

impl std::ops::Index<NodeId> for Document {
    type Output = Node;

    #[inline]
    fn index(&self, id: NodeId) -> &Node {
        &self.nodes[id.get()]
    }
}

impl std::ops::IndexMut<NodeId> for Document {
    #[inline]
    fn index_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.get()]
    }
}

#[cfg(test)]
mod tests {
    use super::{super::node::ElementData, *};
    use html5ever::{local_name, namespace_url, ns, QualName};
    use indexmap::IndexMap;
    use test_case::test_case;

    fn new_element() -> NodeData {
        NodeData::Element {
            element: ElementData::new(QualName::new(None, ns!(), local_name!("span")), vec![]),
            inlining_ignored: false,
        }
    }

    fn roundtrip(bytes: &[u8]) -> Vec<u8> {
        let mut buffer = Vec::new();
        Document::parse_with_options(bytes, 0, InliningMode::Document)
            .serialize(
                &mut buffer,
                IndexMap::default(),
                false,
                false,
                InliningMode::Document,
            )
            .expect("Failed to serialize");
        buffer
    }

    #[test]
    fn test_collect_styles() {
        let doc = Document::parse_with_options(
            r"
<head>
  <style>h1 { color:blue; }</style>
  <style>h1 { color:red }</style>
  <style data-css-inline='ignore'>h1 { color:yellow; }</style>
</head>"
                .as_bytes(),
            0,
            InliningMode::Document,
        );
        let styles = doc.styles().collect::<Vec<_>>();
        assert_eq!(styles.len(), 2);
        assert_eq!(styles[0], "h1 { color:blue; }");
        assert_eq!(styles[1], "h1 { color:red }");
    }

    #[test]
    fn test_collect_stylesheets() {
        let doc = Document::parse_with_options(
            r"
<head>
  <link href='styles1.css' rel='stylesheet' type='text/css'>
  <link href='styles2.css' rel='stylesheet' type='text/css'>
  <link href='' rel='stylesheet' type='text/css'>
  <link href='styles3.css' rel='stylesheet' type='text/css' data-css-inline='ignore'>
</head>"
                .as_bytes(),
            0,
            InliningMode::Document,
        );
        let links = doc.stylesheets().collect::<Vec<_>>();
        assert_eq!(links.len(), 2);
        assert_eq!(links[0], "styles1.css");
        assert_eq!(links[1], "styles2.css");
    }

    #[test]
    fn test_insert_before() {
        let mut doc = Document::with_capacity(0);

        let node1_id = doc.push_node(new_element());
        let node2_id = doc.push_node(new_element());
        let new_node_id = doc.push_node(new_element());

        let document_id = NodeId::document_id();
        doc.append(document_id, node1_id);
        doc.append(document_id, node2_id);

        doc.insert_before(node2_id, new_node_id);

        assert_eq!(doc[node2_id].previous_sibling, Some(new_node_id));
        assert_eq!(doc[new_node_id].next_sibling, Some(node2_id));
    }

    #[test]
    fn test_append() {
        let mut doc = Document::with_capacity(0);

        let node1_id = doc.push_node(new_element());
        let node2_id = doc.push_node(new_element());

        let document_id = NodeId::document_id();
        doc.append(document_id, node1_id);
        doc.append(document_id, node2_id);

        assert_eq!(doc[document_id].last_child, Some(node2_id));
        assert_eq!(doc[node1_id].next_sibling, Some(node2_id));
        assert_eq!(doc[node2_id].previous_sibling, Some(node1_id));
    }

    #[test_case(b"<!DOCTYPE html><html><head><title>Title of the document</title></head><body></body></html>")]
    #[test_case(b"<!DOCTYPE html><html><head><title>Title of the document</title></head><body><hr></body></html>")]
    fn test_roundtrip(input: &[u8]) {
        assert_eq!(roundtrip(input), input);
    }

    #[test]
    fn test_ignore_children() {
        assert_eq!(roundtrip(b"<!DOCTYPE html><html><head><title>Title of the document</title></head><body><hr><hr></hr></hr></body></html>"), b"<!DOCTYPE html><html><head><title>Title of the document</title></head><body><hr><hr></body></html>");
    }

    #[test]
    fn test_pseudo_class() {
        let output = roundtrip(b"<!DOCTYPE html><html><head><title>Title of the document</title><style>h1:hover { color:blue; }</style></head><body><h1>Hello world!</h1></body></html>");
        assert_eq!(output, b"<!DOCTYPE html><html><head><title>Title of the document</title></head><body><h1>Hello world!</h1></body></html>");
    }

    #[test]
    fn test_comment() {
        let output = roundtrip(b"<html><head><title>Title of the document</title></head><body><!--TTT--></body></html>");
        assert_eq!(output, b"<html><head><title>Title of the document</title></head><body><!--TTT--></body></html>");
    }

    #[test]
    fn test_debug() {
        let doc =
            Document::parse_with_options(b"<html><body></body></html>", 0, InliningMode::Document);
        assert_eq!(format!("{doc:?}"), "Document { nodes: [Node { parent: None, next_sibling: None, previous_sibling: None, first_child: None, last_child: None, data: Document }, Node { parent: None, next_sibling: None, previous_sibling: None, first_child: Some(NodeId(2)), last_child: Some(NodeId(2)), data: Document }, Node { parent: Some(NodeId(1)), next_sibling: None, previous_sibling: None, first_child: Some(NodeId(3)), last_child: Some(NodeId(4)), data: Element { element: ElementData { name: QualName { prefix: None, ns: Atom('http://www.w3.org/1999/xhtml' type=static), local: Atom('html' type=static) }, attributes: Attributes { attributes: [], class: None } }, inlining_ignored: false } }, Node { parent: Some(NodeId(2)), next_sibling: Some(NodeId(4)), previous_sibling: None, first_child: None, last_child: None, data: Element { element: ElementData { name: QualName { prefix: None, ns: Atom('http://www.w3.org/1999/xhtml' type=static), local: Atom('head' type=static) }, attributes: Attributes { attributes: [], class: None } }, inlining_ignored: false } }, Node { parent: Some(NodeId(2)), next_sibling: None, previous_sibling: Some(NodeId(3)), first_child: None, last_child: None, data: Element { element: ElementData { name: QualName { prefix: None, ns: Atom('http://www.w3.org/1999/xhtml' type=static), local: Atom('body' type=static) }, attributes: Attributes { attributes: [], class: None } }, inlining_ignored: false } }], styles: [], linked_stylesheets: [], .. }");
    }

    #[test]
    fn test_edit_document() {
        let mut doc = Document::parse_with_options(
            b"<html><body><a></a><b></b></body></html>",
            0,
            InliningMode::Document,
        );
        let a_id = NodeId::new(5);
        assert_eq!(
            doc[a_id].as_element().expect("Element does not exist").name,
            QualName::new(None, ns!(html), local_name!("a"))
        );
        let b_id = NodeId::new(6);
        assert_eq!(
            doc[b_id].as_element().expect("Element does not exist").name,
            QualName::new(None, ns!(html), local_name!("b"))
        );
        // `a` is the previous sibling of `b`
        // `b` is the next sibling of `a`
        assert_eq!(doc[b_id].previous_sibling, Some(a_id));
        assert_eq!(doc[a_id].next_sibling, Some(b_id));
        // Detach `b`, so it has no previous sibling
        // And `a` has not next sibling
        doc.detach(b_id);
        assert_eq!(doc[b_id].previous_sibling, None);
        assert_eq!(doc[a_id].next_sibling, None);
        let head_id = NodeId::new(3);
        let body_id = NodeId::new(4);
        // Detach `head`, so previous sibling of `body` is empty
        doc.detach(head_id);
        assert_eq!(doc[body_id].next_sibling, None);
    }
}
