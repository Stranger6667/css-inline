use super::{
    document::Document,
    element::Element,
    node::{NodeData, NodeId},
    selectors::{ParseError, SelectorAnchor, Selectors},
    Specificity,
};
use selectors::context::SelectorCaches;

/// Compile selectors from a string and create an element iterator that yields elements matching these selectors.
#[inline]
pub(crate) fn select<'a, 'c>(
    document: &'a Document,
    selectors: &str,
    caches: &'c mut SelectorCaches,
) -> Result<Select<'a, 'c>, ParseError> {
    Selectors::compile(selectors).map(|selectors| {
        // Only use indexes if they were built during parsing
        let source = if !document.has_indexes() {
            document.elements.iter()
        } else if !selectors
            .iter()
            .any(|selector| document.has_required_names(selector))
        {
            [].iter()
        } else {
            match selectors.anchor() {
                SelectorAnchor::Id(id) => document.get_by_id(id.as_inner()).iter(),
                SelectorAnchor::Class(class) => document.get_by_class(class.as_inner()).iter(),
                SelectorAnchor::Tag(tag) => document.get_by_tag(tag.as_inner()).iter(),
                SelectorAnchor::Attribute(name) => {
                    document.get_by_attribute(name.as_inner()).iter()
                }
                SelectorAnchor::None => document.elements.iter(),
            }
        };
        Select {
            document,
            caches,
            source,
            selectors,
        }
    })
}

/// An element iterator adaptor that yields elements matching given selectors.
pub(crate) struct Select<'a, 'c> {
    document: &'a Document,
    caches: &'c mut SelectorCaches,
    /// Candidate elements: all of them, or an indexed subset.
    source: std::slice::Iter<'a, NodeId>,
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
        for &element_id in self.source.by_ref() {
            let NodeData::Element { element, .. } = &self.document[element_id].data else {
                unreachable!("Element ids always point to element nodes")
            };
            let element = Element::new(self.document, element_id, element);
            for selector in self.selectors.iter() {
                if element.matches(selector, self.caches) {
                    return Some(element);
                }
            }
        }
        None
    }
}
