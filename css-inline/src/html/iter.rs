use super::{
    document::Document,
    element::Element,
    node::NodeId,
    selectors::{ParseError, Selectors},
    Specificity,
};

/// Compile selectors from a string and create an element iterator that yields elements matching these selectors.
#[inline]
pub(crate) fn select<'a, 'b>(
    document: &'a Document,
    selectors: &'b str,
) -> Result<Select<'a>, ParseError<'b>> {
    Selectors::compile(selectors).map(|selectors| Select {
        traverse: Traverse {
            document,
            current: Some(NodeId::document_id()),
        },
        selectors,
    })
}

/// An internal iterator that traverses a document in tree order.
struct Traverse<'a> {
    document: &'a Document,
    // Current node being processed
    current: Option<NodeId>,
}

impl<'a> Iterator for Traverse<'a> {
    type Item = Element<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Loop until we either run out of nodes or find an element node
        loop {
            if let Some(current) = self.current {
                // Advance to the next node in tree order
                self.current = self.document.next_in_tree_order(current);
                // If the current node is an element node, return it, else continue with the loop
                if let Some(element) = self.document.as_element(current) {
                    return Some(element);
                } else {
                    continue;
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
    traverse: Traverse<'a>,
    /// The selectors to be matched.
    selectors: Selectors,
}

impl<'a> Select<'a> {
    /// Specificity of the first selector in the list of selectors.
    #[inline]
    pub(crate) fn specificity(&self) -> Specificity {
        self.selectors.0[0].specificity()
    }
}

impl<'a> Iterator for Select<'a> {
    type Item = Element<'a>;

    #[inline]
    fn next(&mut self) -> Option<Element<'a>> {
        // Filter the underlying iterator to only return elements that match any of the selectors
        self.traverse
            .by_ref()
            .find(|element| self.selectors.iter().any(|s| element.matches(s)))
    }
}
