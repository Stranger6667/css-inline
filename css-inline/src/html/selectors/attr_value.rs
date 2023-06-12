use cssparser::ToCss;
use html5ever::tendril::StrTendril;
use std::{fmt, fmt::Write};

/// To use `StrTendril` in selectors, we need to implement `ToCss` on a wrapper.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AttrValue(StrTendril);

impl ToCss for AttrValue {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
    where
        W: Write,
    {
        write!(cssparser::CssStringWriter::new(dest), "{}", &self.0)
    }
}

impl<'a> From<&'a str> for AttrValue {
    fn from(string: &'a str) -> Self {
        Self(string.into())
    }
}

impl AsRef<str> for AttrValue {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}
