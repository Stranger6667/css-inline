use super::{
    document::Document,
    element::Element,
    node::NodeId,
    selectors::{ParseError, Selectors},
    Specificity,
};

pub(crate) fn select<'a, 'b>(
    document: &'a Document,
    selectors: &'b str,
) -> Result<Select<'a>, ParseError<'b>> {
    Selectors::compile(selectors).map(|selectors| Select {
        iter: Iter {
            document,
            current: Some(NodeId::document_id()),
        },
        selectors,
    })
}

struct Iter<'a> {
    document: &'a Document,
    current: Option<NodeId>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Element<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(current) = self.current {
                self.current = self.document.next_in_tree_order(current);
                if let Some(element) = self.document.as_element(current) {
                    return Some(element);
                } else {
                    continue;
                }
            } else {
                return None;
            }
        }
    }
}

/// An element iterator adaptor that yields elements matching given selectors.
pub(crate) struct Select<'a> {
    iter: Iter<'a>,
    /// The selectors to be matched.
    selectors: Selectors,
}

impl<'a> Select<'a> {
    pub(crate) fn specificity(&self) -> Specificity {
        self.selectors.0[0].specificity()
    }
}

impl<'a> Iterator for Select<'a> {
    type Item = Element<'a>;

    #[inline]
    fn next(&mut self) -> Option<Element<'a>> {
        self.iter
            .by_ref()
            .find(|element| self.selectors.iter().any(|s| element.matches(s)))
    }
}
