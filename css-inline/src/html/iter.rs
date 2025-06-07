use super::{
    document::Document,
    element::Element,
    node::{NodeData, NodeId},
    selectors::{ParseError, Selectors},
    Specificity,
};
use selectors::context::SelectorCaches;

/// Compile selectors from a string and create an element iterator that yields elements matching these selectors.
#[inline]
pub(crate) fn select<'a, 'b, 'c>(
    document: &'a Document,
    selectors: &'b str,
    caches: &'c mut SelectorCaches,
) -> Result<Select<'a, 'c>, ParseError<'b>> {
    Selectors::compile(selectors).map(|selectors| Select {
        document,
        caches,
        iter: document.elements.iter(),
        selectors,
    })
}

/// An element iterator adaptor that yields elements matching given selectors.
pub(crate) struct Select<'a, 'c> {
    document: &'a Document,
    caches: &'c mut SelectorCaches,
    iter: std::slice::Iter<'a, NodeId>,
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
        for element_id in self.iter.by_ref() {
            let NodeData::Element { element, .. } = &self.document[*element_id].data else {
                unreachable!("Element ids always point to element nodes")
            };
            let element = Element::new(self.document, *element_id, element);
            for selector in self.selectors.iter() {
                if element.matches(selector, self.caches) {
                    return Some(element);
                }
            }
        }
        None
    }
}
