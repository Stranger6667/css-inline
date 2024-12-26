use super::{
    document::Document,
    element::Element,
    node::{NodeData, NodeId},
    selectors::{ParseError, Selectors},
    Specificity,
};
use selectors::NthIndexCache;
use std::iter::Zip;

/// Compile selectors from a string and create an element iterator that yields elements matching these selectors.
#[inline]
pub(crate) fn select<'a, 'b, 'c>(
    document: &'a Document,
    selectors: &'b str,
    caches: &'c mut [NthIndexCache],
) -> Result<Select<'a, 'c>, ParseError<'b>> {
    Selectors::compile(selectors).map(|selectors| Select {
        document,
        iter: document.elements.iter().zip(caches.iter_mut()),
        selectors,
    })
}

/// An element iterator adaptor that yields elements matching given selectors.
pub(crate) struct Select<'a, 'c> {
    document: &'a Document,
    iter: Zip<std::slice::Iter<'a, NodeId>, std::slice::IterMut<'c, NthIndexCache>>,
    /// The selectors to be matched.
    selectors: Selectors,
}

impl Select<'_, '_> {
    /// Specificity of the first selector in the list of selectors.
    #[inline]
    pub(crate) fn specificity(&self) -> Specificity {
        Specificity::new(self.selectors.0[0].specificity())
    }
}

impl<'a> Iterator for Select<'a, '_> {
    type Item = Element<'a>;

    #[inline]
    fn next(&mut self) -> Option<Element<'a>> {
        for (element_id, cache) in self.iter.by_ref() {
            let NodeData::Element { element, .. } = &self.document[*element_id].data else {
                unreachable!("Element ids always point to element nodes")
            };
            let element = Element::new(self.document, *element_id, element);
            for selector in self.selectors.iter() {
                if element.matches(selector, cache) {
                    return Some(element);
                }
            }
        }
        None
    }
}
