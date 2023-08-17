use html5ever::{local_name, namespace_url, ns, tendril::StrTendril, QualName};
use std::collections::BTreeMap;

/// A collection of HTML attributes.
#[derive(Debug)]
pub(crate) struct Attributes {
    /// Attribute names and their respective values.
    pub(crate) map: BTreeMap<QualName, StrTendril>,
    pub(crate) class: Option<StrTendril>,
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
        let mut class = None;
        let mut map = BTreeMap::new();
        for attr in attributes {
            if attr.name.local == local_name!("class") {
                class = Some(attr.value);
            } else {
                map.insert(attr.name, attr.value);
            }
        }
        Attributes { map, class }
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
}
