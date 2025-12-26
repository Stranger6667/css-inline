use super::{
    document::Document,
    element::Element,
    node::{NodeData, NodeId},
    selectors::{ParseError, SelectorAnchor, Selectors},
    Specificity,
};
use selectors::context::SelectorCaches;

/// Source of elements to iterate over.
/// Allows using different indexes for faster lookup.
enum ElementSource<'a> {
    /// Iterate over a slice of elements (all elements or indexed subset).
    Slice(std::slice::Iter<'a, NodeId>),
    /// Single element from ID lookup.
    Single(Option<NodeId>),
}

impl ElementSource<'_> {
    #[inline]
    fn next(&mut self) -> Option<NodeId> {
        match self {
            ElementSource::Slice(iter) => iter.next().copied(),
            ElementSource::Single(opt) => opt.take(),
        }
    }
}

/// Compile selectors from a string and create an element iterator that yields elements matching these selectors.
#[inline]
pub(crate) fn select<'a, 'b, 'c>(
    document: &'a Document,
    selectors: &'b str,
    caches: &'c mut SelectorCaches,
) -> Result<Select<'a, 'c>, ParseError<'b>> {
    Selectors::compile(selectors).map(|selectors| {
        // Only use indexes if they were built during parsing
        let source = if document.has_indexes() {
            match selectors.anchor() {
                SelectorAnchor::Id(id) => ElementSource::Single(document.get_by_id(id.as_inner())),
                SelectorAnchor::Class(class) => {
                    ElementSource::Slice(document.get_by_class(class.as_inner()).iter())
                }
                SelectorAnchor::Tag(tag) => {
                    ElementSource::Slice(document.get_by_tag(tag.as_inner()).iter())
                }
                SelectorAnchor::None => ElementSource::Slice(document.elements.iter()),
            }
        } else {
            ElementSource::Slice(document.elements.iter())
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
    source: ElementSource<'a>,
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
        while let Some(element_id) = self.source.next() {
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
