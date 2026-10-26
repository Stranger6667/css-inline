use super::{
    document::{Document, NameSet},
    element::Element,
    node::{NodeData, NodeId},
    selectors::{ancestor_names, ParseError, SelectorAnchor, Selectors},
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
            ElementSource::Slice(document.elements.iter())
        } else if !selectors
            .iter()
            .any(|selector| document.has_required_names(selector))
        {
            ElementSource::Slice([].iter())
        } else {
            match selectors.anchor() {
                SelectorAnchor::Id(id) => {
                    ElementSource::Slice(document.get_by_id(id.as_inner()).iter())
                }
                SelectorAnchor::Class(class) => {
                    ElementSource::Slice(document.get_by_class(class.as_inner()).iter())
                }
                SelectorAnchor::Tag(tag) => {
                    ElementSource::Slice(document.get_by_tag(tag.as_inner()).iter())
                }
                SelectorAnchor::Attribute(name) => {
                    ElementSource::Slice(document.get_by_attribute(name.as_inner()).iter())
                }
                SelectorAnchor::Root => {
                    ElementSource::Roots(document[NodeId::document_id()].first_child)
                }
                SelectorAnchor::None => ElementSource::Slice(document.elements.iter()),
            }
        };
        // Only large documents, as building the ancestor names costs a tree pass
        let ancestors = match selectors.0.as_slice() {
            [single] if document.has_indexes() => {
                Some(ancestor_names(single)).filter(|names| !names.is_empty())
            }
            _ => None,
        };
        Select {
            document,
            caches,
            source,
            selectors,
            ancestors,
        }
    })
}

/// Candidate elements for selector matching.
enum ElementSource<'a> {
    /// All elements, or an indexed subset.
    Slice(std::slice::Iter<'a, NodeId>),
    /// Children of the document node, starting from this one.
    Roots(Option<NodeId>),
}

/// An element iterator adaptor that yields elements matching given selectors.
pub(crate) struct Select<'a, 'c> {
    document: &'a Document,
    caches: &'c mut SelectorCaches,
    source: ElementSource<'a>,
    /// The selectors to be matched.
    selectors: Selectors,
    /// Names that some ancestor of every matching element has, if any.
    ancestors: Option<NameSet>,
}

impl<'a> Select<'a, '_> {
    /// Specificity of the first selector in the list of selectors.
    #[inline]
    pub(crate) fn specificity(&self) -> Specificity {
        Specificity::new(self.selectors.0[0].specificity())
    }

    /// The next candidate that passes `may_match` and matches the selectors.
    #[inline]
    fn find_match(&mut self, may_match: impl Fn(&Document, NodeId) -> bool) -> Option<Element<'a>> {
        loop {
            let element_id = match &mut self.source {
                ElementSource::Slice(iter) => *iter.next()?,
                // Skip non-element children of the document node, e.g. comments or doctype
                ElementSource::Roots(next) => loop {
                    let node_id = (*next)?;
                    let node = &self.document[node_id];
                    *next = node.next_sibling;
                    if matches!(node.data, NodeData::Element { .. }) {
                        break node_id;
                    }
                },
            };
            let NodeData::Element { element, .. } = &self.document[element_id].data else {
                unreachable!("Element ids always point to element nodes")
            };
            if !may_match(self.document, element_id) {
                continue;
            }
            let element = Element::new(self.document, element_id, element);
            for selector in self.selectors.iter() {
                if element.matches(selector, self.caches) {
                    return Some(element);
                }
            }
        }
    }
}

impl<'a> Iterator for Select<'a, '_> {
    type Item = Element<'a>;

    #[inline]
    fn next(&mut self) -> Option<Element<'a>> {
        // Separate loops keep the unfiltered one as lean as without the filter
        match self.ancestors {
            None => self.find_match(|_, _| true),
            Some(required) => self.find_match(|document, element_id| {
                document
                    .ancestor_names(element_id)
                    .is_none_or(|names| names.contains_all(&required))
            }),
        }
    }
}
