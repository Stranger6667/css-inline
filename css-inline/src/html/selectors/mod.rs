mod attr_value;
mod local_name;
mod parser;
mod pseudo_classes;
mod pseudo_elements;
mod selector_impl;

pub(super) use attr_value::AttrValue;
pub(super) use local_name::LocalName;
pub(super) use pseudo_classes::PseudoClass;
pub(super) use pseudo_elements::PseudoElement;
pub(super) use selector_impl::InlinerSelectors;

use selectors::{
    parser::{Selector as GenericSelector, SelectorParseErrorKind},
    SelectorList,
};

/// A pre-compiled CSS Selector.
pub(crate) struct Selector(GenericSelector<InlinerSelectors>);

/// A pre-compiled list of CSS Selectors.
pub(crate) struct Selectors(pub(crate) Vec<Selector>);

/// The specificity of a selector.
/// Determines precedence in the cascading algorithm.
/// When equal, a rule later in source order takes precedence.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Specificity(u32);

pub(crate) type ParseError<'i> = cssparser::ParseError<'i, SelectorParseErrorKind<'i>>;

/// Parse CSS selectors into `SelectorList`.
fn parse(selectors: &str) -> Result<SelectorList<InlinerSelectors>, ParseError<'_>> {
    let mut input = cssparser::ParserInput::new(selectors);
    let parser = &mut cssparser::Parser::new(&mut input);
    SelectorList::parse(&parser::SelectorParser, parser)
}

impl Selectors {
    /// Compile a list of selectors.
    #[inline]
    pub(super) fn compile(selectors: &str) -> Result<Selectors, ParseError<'_>> {
        parse(selectors).map(|list| Selectors(list.0.into_iter().map(Selector).collect()))
    }

    /// Iterator over selectors.
    #[inline]
    pub(super) fn iter(&self) -> impl Iterator<Item = &Selector> {
        self.0.iter()
    }
}

impl Selector {
    /// Inner selector value.
    pub(crate) fn inner_value(&self) -> &GenericSelector<InlinerSelectors> {
        &self.0
    }
    /// Specificity of this selector.
    pub(crate) fn specificity(&self) -> Specificity {
        Specificity(self.0.specificity())
    }
}
