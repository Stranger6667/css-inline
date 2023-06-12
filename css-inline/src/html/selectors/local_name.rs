use cssparser::ToCss;
use std::fmt::Write;

/// `LocalName` type wraps `html5ever::LocalName` to extend it by implementing the `ToCss` trait,
/// which is needed for selectors implementation.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub(crate) struct LocalName(html5ever::LocalName);

impl LocalName {
    /// Returns the inner `html5ever::LocalName`.
    pub(crate) fn into_inner(self) -> html5ever::LocalName {
        self.0
    }
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl PartialEq<LocalName> for html5ever::LocalName {
    fn eq(&self, other: &LocalName) -> bool {
        self.eq(&other.0)
    }
}

impl<'a> From<&'a str> for LocalName {
    fn from(value: &'a str) -> Self {
        LocalName(html5ever::LocalName::from(value))
    }
}

impl ToCss for LocalName {
    fn to_css<W>(&self, dest: &mut W) -> std::fmt::Result
    where
        W: Write,
    {
        // Write the `LocalName` to the destination, which is the CSS syntax.
        dest.write_str(self.0.as_ref())
    }
}
