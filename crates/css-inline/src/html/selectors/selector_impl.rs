use super::{
    attr_value::AttrValue, local_name::LocalName, pseudo_classes::PseudoClass,
    pseudo_elements::PseudoElement,
};
use html5ever::Namespace;
use selectors::SelectorImpl;

/// CSS selectors implementation. It is needed to parametrize the parser implementation in regards
/// of pseudo-classes and elements.
#[derive(Debug, Clone)]
pub(crate) struct InlinerSelectors;

impl SelectorImpl for InlinerSelectors {
    type ExtraMatchingData<'a> = std::marker::PhantomData<&'a ()>;
    type AttrValue = AttrValue;
    type Identifier = LocalName;
    type LocalName = LocalName;
    type NamespaceUrl = Namespace;
    type NamespacePrefix = LocalName;
    type BorrowedNamespaceUrl = Namespace;
    type BorrowedLocalName = LocalName;
    type NonTSPseudoClass = PseudoClass;
    type PseudoElement = PseudoElement;
}
