use html5ever::{local_name, namespace_url, ns, tendril::StrTendril, QualName};
use std::collections::BTreeMap;

/// A collection of HTML attributes.
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Attributes {
    /// Attribute names and their respective values.
    pub(crate) map: BTreeMap<QualName, StrTendril>,
}

pub(crate) const CSS_INLINE_ATTRIBUTE: &str = "data-css-inline";

/// Whether an HTML element's attributes contain the 'data-css-inline' flag set to 'ignore'.
pub(super) fn should_ignore(attributes: &[html5ever::Attribute]) -> bool {
    attributes
        .iter()
        .any(|a| a.name.local == *CSS_INLINE_ATTRIBUTE && a.value == "ignore".into())
}

impl Attributes {
    pub(crate) fn new(attributes: Vec<html5ever::Attribute>) -> Attributes {
        Attributes {
            map: attributes
                .into_iter()
                .map(|attr| (attr.name, attr.value))
                .collect(),
        }
    }

    /// Checks if the attributes map contains a given local name.
    pub(crate) fn contains(&self, local: html5ever::LocalName) -> bool {
        self.map.contains_key(&QualName::new(None, ns!(), local))
    }

    /// Get the value of the attribute with the given local name, if it exists.
    pub(crate) fn get(&self, local: html5ever::LocalName) -> Option<&str> {
        self.map
            .get(&QualName::new(None, ns!(), local))
            .map(|value| &**value)
    }

    pub(crate) fn get_style_mut(&mut self) -> Option<&mut StrTendril> {
        self.map
            .get_mut(&QualName::new(None, ns!(), local_name!("style")))
    }

    pub(crate) fn set_style(&mut self, style: String) {
        self.map.insert(
            QualName::new(None, ns!(), local_name!("style")),
            style.into(),
        );
    }
}
