use super::{
    document::Document,
    element::Element,
    node::{Node, NodeData, NodeId},
    selectors::{ParseError, Selectors},
    Specificity,
};
use std::iter::{Enumerate, Skip};

/// Compile selectors from a string and create an element iterator that yields elements matching these selectors.
#[inline]
pub(crate) fn select<'a, 'b>(
    document: &'a Document,
    selectors: &'b str,
) -> Result<Select<'a>, ParseError<'b>> {
    Selectors::compile(selectors).map(|selectors| Select {
        elements: Elements {
            document,
            // Skip the dummy & document nodes
            iter: document.nodes.iter().enumerate().skip(2),
        },
        selectors,
    })
}

/// An internal iterator that traverses a document.
struct Elements<'a> {
    document: &'a Document,
    iter: Skip<Enumerate<std::slice::Iter<'a, Node>>>,
}

impl<'a> Iterator for Elements<'a> {
    type Item = Element<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Loop until we either run out of nodes or find an element node
        loop {
            if let Some((id, node)) = self.iter.next() {
                // If the current node is an element node, return it, else continue with the loop
                if let NodeData::Element { element, .. } = &node.data {
                    return Some(Element::new(self.document, NodeId::new(id), element));
                }
            } else {
                // No more elements in the document
                return None;
            }
        }
    }
}

/// An element iterator adaptor that yields elements matching given selectors.
pub(crate) struct Select<'a> {
    elements: Elements<'a>,
    /// The selectors to be matched.
    selectors: Selectors,
}

impl<'a> Select<'a> {
    /// Specificity of the first selector in the list of selectors.
    #[inline]
    pub(crate) fn specificity(&self) -> Specificity {
        Specificity::new(self.selectors.0[0].specificity())
    }
}

impl<'a> Iterator for Select<'a> {
    type Item = Element<'a>;

    #[inline]
    fn next(&mut self) -> Option<Element<'a>> {
        // Filter the underlying iterator to only return elements that match any of the selectors
        for element in self.elements.by_ref() {
            for selector in self.selectors.iter() {
                if element.matches(selector) {
                    return Some(element);
                }
            }
        }
        None
    }
}
