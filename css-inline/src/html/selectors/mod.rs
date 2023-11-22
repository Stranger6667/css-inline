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
    parser::{ParseRelative, Selector as GenericSelector, SelectorParseErrorKind},
    SelectorList,
};
use smallvec::SmallVec;

/// A pre-compiled CSS Selector.
pub(crate) type Selector = GenericSelector<InlinerSelectors>;

/// A pre-compiled list of CSS Selectors.
pub(crate) struct Selectors(pub(crate) SmallVec<[Selector; 1]>);

/// The specificity of a selector.
/// Determines precedence in the cascading algorithm.
/// When equal, a rule later in source order takes precedence.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Specificity(u32);

impl Specificity {
    #[inline]
    pub(crate) fn new(value: u32) -> Specificity {
        Specificity(value)
    }
}

pub(crate) type ParseError<'i> = cssparser::ParseError<'i, SelectorParseErrorKind<'i>>;

/// Parse CSS selectors into `SelectorList`.
fn parse(selectors: &str) -> Result<SelectorList<InlinerSelectors>, ParseError<'_>> {
    let mut input = cssparser::ParserInput::new(selectors);
    let parser = &mut cssparser::Parser::new(&mut input);
    SelectorList::parse(&parser::SelectorParser, parser, ParseRelative::No)
}

impl Selectors {
    /// Compile a list of selectors.
    #[inline]
    pub(super) fn compile(selectors: &str) -> Result<Selectors, ParseError<'_>> {
        parse(selectors).map(|list| Selectors(list.0))
    }

    /// Iterator over selectors.
    #[inline]
    pub(super) fn iter(&self) -> impl Iterator<Item = &Selector> {
        self.0.iter()
    }
}
