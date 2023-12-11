use super::{
    document::Document,
    element::Element,
    node::{NodeData, NodeId},
    selectors::{ParseError, Selectors},
    Specificity,
};
use selectors::NthIndexCache;
use std::cell::RefCell;

/// Compile selectors from a string and create an element iterator that yields elements matching these selectors.
#[inline]
pub(crate) fn select<'a, 'b>(
    document: &'a Document,
    selectors: &'b str,
) -> Result<Select<'a>, ParseError<'b>> {
    Selectors::compile(selectors).map(|selectors| Select {
        document,
        iter: document.elements.iter(),
        selectors,
    })
}

/// An element iterator adaptor that yields elements matching given selectors.
pub(crate) struct Select<'a> {
    document: &'a Document,
    iter: std::slice::Iter<'a, (NodeId, RefCell<NthIndexCache>)>,
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
        for (element_id, cache) in self.iter.by_ref() {
            let NodeData::Element { element, .. } = &self.document[*element_id].data else {
                unreachable!("Element ids always point to element nodes")
            };
            let element = Element::new(self.document, *element_id, element);
            let mut cache = cache.borrow_mut();
            for selector in self.selectors.iter() {
                if element.matches(selector, &mut cache) {
                    return Some(element);
                }
            }
        }
        None
    }
}
