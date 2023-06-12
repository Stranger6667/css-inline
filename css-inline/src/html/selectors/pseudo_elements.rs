use super::selector_impl::InlinerSelectors;
use cssparser::ToCss;
use std::fmt;

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) enum PseudoElement {}

impl ToCss for PseudoElement {
    fn to_css<W>(&self, _dest: &mut W) -> fmt::Result
    where
        W: fmt::Write,
    {
        match *self {}
    }
}

impl selectors::parser::PseudoElement for PseudoElement {
    type Impl = InlinerSelectors;
}
