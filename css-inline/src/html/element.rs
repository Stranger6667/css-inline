use super::{
    attributes::Attributes,
    document::Document,
    node::{ElementData, NodeData, NodeId},
    selectors::{AttrValue, InlinerSelectors, LocalName, PseudoClass, PseudoElement, Selector},
};
use html5ever::{local_name, namespace_url, ns, Namespace, QualName};
use selectors::{
    attr::{AttrSelectorOperation, CaseSensitivity, NamespaceConstraint},
    context::QuirksMode,
    matching, NthIndexCache, OpaqueElement,
};
use std::cmp::Ordering;

/// A reference to an element node in a document.
/// This structure is necessary for accessing and iterating over other nodes in the tree.
#[derive(Debug, Clone)]
pub(crate) struct Element<'a> {
    /// Reference to the original document.
    pub(crate) document: &'a Document,
    /// The unique identifier of the node in the document.
    pub(crate) node_id: NodeId,
    /// The specific data associated with the element.
    data: &'a ElementData,
}

impl<'a> Element<'a> {
    pub(crate) fn new(
        document: &'a Document,
        node_id: NodeId,
        data: &'a ElementData,
    ) -> Element<'a> {
        Element {
            document,
            node_id,
            data,
        }
    }
    /// Qualified name of the element.
    #[inline]
    pub(crate) fn name(&self) -> &QualName {
        &self.data.name
    }
    /// Attributes of the element.
    #[inline]
    pub(crate) fn attributes(&self) -> &Attributes {
        &self.data.attributes
    }
    /// A reference to the element data.
    #[inline]
    pub(crate) fn data(&self) -> &'a ElementData {
        self.data
    }
    /// ID of the parent node of the element, if it exists.
    #[inline]
    pub(crate) fn parent(&self) -> Option<NodeId> {
        self.document[self.node_id].parent
    }
    /// The parent element of the current element, if it exists.
    /// This function will return `None` if the parent node is not an element.
    pub(crate) fn parent_element(&self) -> Option<Element<'a>> {
        self.parent()
            .and_then(|node_id| self.document.as_element(node_id))
    }
    fn previous_sibling_element(&self) -> Option<Element<'a>> {
        let mut node = &self.document[self.node_id];
        loop {
            if let Some(previous_sibling_id) = node.previous_sibling {
                let previous_sibling = &self.document[previous_sibling_id];
                if let NodeData::Element { element, .. } = &previous_sibling.data {
                    return Some(Element::new(self.document, previous_sibling_id, element));
                }
                node = previous_sibling;
            } else {
                // Node has no previous sibling at all
                return None;
            }
        }
    }
    fn next_sibling_element(&self) -> Option<Element<'a>> {
        let mut node = &self.document[self.node_id];
        loop {
            if let Some(next_sibling_id) = node.next_sibling {
                let next_sibling = &self.document[next_sibling_id];
                if let NodeData::Element { element, .. } = &next_sibling.data {
                    return Some(Element::new(self.document, next_sibling_id, element));
                }
                node = next_sibling;
            } else {
                // Node has no next sibling at all
                return None;
            }
        }
    }
    pub(crate) fn matches(&self, selector: &Selector, cache: &mut NthIndexCache) -> bool {
        let mut context = matching::MatchingContext::new(
            matching::MatchingMode::Normal,
            None,
            cache,
            QuirksMode::NoQuirks,
            matching::NeedsSelectorFlags::No,
            matching::IgnoreNthChildForInvalidation::No,
        );
        matching::matches_selector(selector, 0, None, self, &mut context)
    }
}

impl selectors::Element for Element<'_> {
    type Impl = InlinerSelectors;

    #[inline]
    fn opaque(&self) -> OpaqueElement {
        OpaqueElement::new(self.data())
    }

    #[inline]
    fn parent_element(&self) -> Option<Self> {
        self.parent_element()
    }

    #[inline]
    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }

    #[inline]
    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }

    #[inline]
    fn is_pseudo_element(&self) -> bool {
        false
    }

    #[inline]
    fn prev_sibling_element(&self) -> Option<Self> {
        self.previous_sibling_element()
    }

    #[inline]
    fn next_sibling_element(&self) -> Option<Self> {
        self.next_sibling_element()
    }

    #[inline]
    fn is_html_element_in_html_document(&self) -> bool {
        self.name().ns == ns!(html)
    }

    #[inline]
    fn has_local_name(&self, name: &LocalName) -> bool {
        self.name().local == *name
    }

    #[inline]
    fn has_namespace(&self, namespace: &Namespace) -> bool {
        self.name().ns == *namespace
    }

    #[inline]
    fn is_same_type(&self, other: &Self) -> bool {
        self.name() == other.name()
    }

    #[inline]
    fn attr_matches(
        &self,
        ns: &NamespaceConstraint<&Namespace>,
        local_name: &LocalName,
        operation: &AttrSelectorOperation<&AttrValue>,
    ) -> bool {
        let attrs = self.attributes();
        match *ns {
            NamespaceConstraint::Any => attrs
                .attributes
                .iter()
                .any(|attr| attr.name.local == *local_name && operation.eval_str(&attr.value)),
            NamespaceConstraint::Specific(_) if attrs.attributes.is_empty() => false,
            NamespaceConstraint::Specific(ns_url) => attrs
                .find(&QualName::new(
                    None,
                    ns_url.clone(),
                    local_name.clone().into_inner(),
                ))
                .map_or(false, |value| operation.eval_str(value)),
        }
    }

    #[allow(clippy::enum_glob_use)]
    fn match_non_ts_pseudo_class(
        &self,
        pseudo: &PseudoClass,
        _context: &mut matching::MatchingContext<'_, InlinerSelectors>,
    ) -> bool {
        use self::PseudoClass::*;
        match *pseudo {
            Active | Focus | Hover | Enabled | Disabled | Checked | Indeterminate | Visited => {
                false
            }
            AnyLink | Link => {
                self.name().ns == ns!(html)
                    && matches!(
                        self.name().local,
                        local_name!("a") | local_name!("area") | local_name!("link")
                    )
                    && self.attributes().contains(local_name!("href"))
            }
        }
    }

    fn match_pseudo_element(
        &self,
        pseudo: &PseudoElement,
        _context: &mut matching::MatchingContext<'_, InlinerSelectors>,
    ) -> bool {
        match *pseudo {}
    }

    #[inline]
    fn is_link(&self) -> bool {
        self.name().ns == ns!(html)
            && matches!(
                self.name().local,
                local_name!("a") | local_name!("area") | local_name!("link")
            )
            && self.attributes().contains(local_name!("href"))
    }

    #[inline]
    fn is_html_slot_element(&self) -> bool {
        false
    }

    #[inline]
    fn has_id(&self, id: &LocalName, case_sensitivity: CaseSensitivity) -> bool {
        self.attributes()
            .get(local_name!("id"))
            .map_or(false, |id_attr| {
                case_sensitivity.eq(id.as_bytes(), id_attr.as_bytes())
            })
    }

    #[inline]
    fn has_class(&self, name: &LocalName, case_sensitivity: CaseSensitivity) -> bool {
        let name = name.as_bytes();

        !name.is_empty()
            && if let Some(class_attr) = &self.attributes().class {
                let class = class_attr.value.as_bytes();
                match class.len().cmp(&name.len()) {
                    Ordering::Less => false,
                    Ordering::Equal => case_sensitivity.eq(class, name),
                    Ordering::Greater => class_attr.has_class(name, case_sensitivity),
                }
            } else {
                false
            }
    }

    #[inline]
    fn imported_part(&self, _: &LocalName) -> Option<LocalName> {
        None
    }

    #[inline]
    fn is_part(&self, _name: &LocalName) -> bool {
        false
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.document
            .children(self.node_id)
            .all(|child| match &self.document[child].data {
                NodeData::Element { .. } => false,
                NodeData::Text { text } => text.is_empty(),
                _ => true,
            })
    }

    #[inline]
    fn is_root(&self) -> bool {
        match self.parent() {
            None => false,
            Some(node_id) => matches!(self.document[node_id].data, NodeData::Document),
        }
    }

    fn first_element_child(&self) -> Option<Self> {
        self.document[self.node_id]
            .first_child
            .and_then(|node_id| self.document.as_element(node_id))
    }

    fn apply_selector_flags(&self, _: matching::ElementSelectorFlags) {}
}
