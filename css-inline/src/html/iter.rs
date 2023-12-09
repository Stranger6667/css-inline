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
        elements: Elements {
            document,
            iter: document.elements.iter(),
        },
        selectors,
    })
}

/// An internal iterator that traverses a document.
struct Elements<'a> {
    document: &'a Document,
    iter: std::slice::Iter<'a, (NodeId, RefCell<NthIndexCache>)>,
}

impl<'a> Iterator for Elements<'a> {
    type Item = (Element<'a>, &'a RefCell<NthIndexCache>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((element_id, cache)) = self.iter.next() {
            let NodeData::Element { element, .. } = &self.document[*element_id].data else {
                unreachable!("Element ids always point to element nodes")
            };
            Some((Element::new(self.document, *element_id, element), cache))
        } else {
            // No more elements in the document
            None
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
        for (element, cache) in self.elements.by_ref() {
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
