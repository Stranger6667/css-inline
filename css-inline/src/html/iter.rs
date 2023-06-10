use super::{
    document::Document,
    element::Element,
    selectors::{ParseError, Selectors},
    Specificity,
};

pub(crate) fn select<'a, 'b>(
    document: &'a Document,
    selectors: &'b str,
) -> Result<Select<'a>, ParseError<'b>> {
    Iter { document }.select(selectors)
}

#[derive(Debug, Clone)]
pub(crate) struct Iter<'a> {
    document: &'a Document,
    // TODO: iterator state
}

impl<'a> Iterator for Iter<'a> {
    type Item = Element<'a>;
    #[inline]
    fn next(&mut self) -> Option<Element<'a>> {
        // TODO: Implement iteration
        None
    }
}

/// An element iterator adaptor that yields elements matching given selectors.
pub(crate) struct Select<'a> {
    iter: Iter<'a>,
    /// The selectors to be matched.
    selectors: Selectors,
}

impl<'a> Iter<'a> {
    pub(super) fn select<'b>(self, selectors: &'b str) -> Result<Select<'a>, ParseError<'b>> {
        Selectors::compile(selectors).map(|s| Select {
            iter: self,
            selectors: s,
        })
    }
}

impl<'a> Select<'a> {
    pub(crate) fn specificity(&self) -> Specificity {
        self.selectors.0[0].specificity()
    }
}

impl<'a> Iterator for Select<'a> {
    type Item = Element<'a>;

    #[inline]
    fn next(&mut self) -> Option<Element<'a>> {
        self.iter
            .by_ref()
            .find(|element| self.selectors.iter().any(|s| element.matches(s)))
    }
}
