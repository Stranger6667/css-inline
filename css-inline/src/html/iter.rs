use super::{
    document::Document,
    element::Element,
    node::{NodeData, NodeId},
    selectors::{ParseError, Selectors},
    Specificity,
};
use crate::html::selectors::InlinerSelectors;
use selectors::matching::MatchingContext;
use std::iter::Zip;

/// Compile selectors from a string and create an element iterator that yields elements matching these selectors.
#[inline]
pub(crate) fn select<'a: 'c, 'b, 'c>(
    document: &'a Document,
    selectors: &'b str,
    matching_contexts: &'c mut [MatchingContext<'a, InlinerSelectors>],
) -> Result<Select<'a, 'c>, ParseError<'b>> {
    Selectors::compile(selectors).map(|selectors| Select {
        elements: Elements {
            document,
            iter: document.elements.iter().zip(matching_contexts.iter_mut()),
        },
        selectors,
    })
}

/// An internal iterator that traverses a document.
struct Elements<'a, 'c> {
    document: &'a Document,
    iter: Zip<
        std::slice::Iter<'a, NodeId>,
        std::slice::IterMut<'c, MatchingContext<'a, InlinerSelectors>>,
    >,
}

impl<'a: 'c, 'c> Iterator for Elements<'a, 'c> {
    type Item = (Element<'a>, &'c mut MatchingContext<'a, InlinerSelectors>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((element_id, context)) = self.iter.next() {
            let NodeData::Element { element, .. } = &self.document[*element_id].data else {
                unreachable!("Element ids always point to element nodes")
            };
            Some((Element::new(self.document, *element_id, element), context))
        } else {
            // No more elements in the document
            None
        }
    }
}

/// An element iterator adaptor that yields elements matching given selectors.
pub(crate) struct Select<'a, 'c> {
    elements: Elements<'a, 'c>,
    /// The selectors to be matched.
    selectors: Selectors,
}

impl<'a: 'c, 'c> Select<'a, 'c> {
    /// Specificity of the first selector in the list of selectors.
    #[inline]
    pub(crate) fn specificity(&self) -> Specificity {
        Specificity::new(self.selectors.0[0].specificity())
    }
}

impl<'a: 'c, 'c> Iterator for Select<'a, 'c> {
    type Item = Element<'a>;

    #[inline]
    fn next(&mut self) -> Option<Element<'a>> {
        // Filter the underlying iterator to only return elements that match any of the selectors
        for (element, context) in self.elements.by_ref() {
            for selector in self.selectors.iter() {
                if element.matches(selector, context) {
                    return Some(element);
                }
            }
        }
        None
    }
}
