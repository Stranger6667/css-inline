use html5ever::{local_name, namespace_url, ns, tendril::StrTendril, QualName};
use rustc_hash::FxHasher;
use selectors::{
    attr::CaseSensitivity,
    bloom::{BloomStorageBool, CountingBloomFilter},
};
use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
};

#[derive(Debug)]
pub(crate) struct Class {
    pub(crate) value: StrTendril,
    pub(crate) cache: CountingBloomFilter<BloomStorageBool>,
}

static SELECTOR_WHITESPACE: &[char] = &[' ', '\t', '\n', '\r', '\x0C'];

impl Class {
    fn new(value: StrTendril) -> Class {
        let mut cache = CountingBloomFilter::<BloomStorageBool>::new();
        for class in value.split(SELECTOR_WHITESPACE) {
            let hash = hash_class_name(class.as_bytes());
            cache.insert_hash(hash);
        }
        Class { value, cache }
    }

    #[inline]
    pub(crate) fn might_have_class(&self, name: &[u8]) -> bool {
        let hash = hash_class_name(name);
        self.cache.might_contain_hash(hash)
    }

    #[inline]
    fn has_class_impl(&self, name: &[u8], case_sensitivity: CaseSensitivity) -> bool {
        for class in self.value.split(SELECTOR_WHITESPACE) {
            if case_sensitivity.eq(class.as_bytes(), name) {
                return true;
            }
        }
        false
    }

    #[inline]
    pub(crate) fn has_class(&self, name: &[u8], case_sensitivity: CaseSensitivity) -> bool {
        match case_sensitivity {
            CaseSensitivity::CaseSensitive => {
                if self.might_have_class(name) {
                    self.has_class_impl(name, case_sensitivity)
                } else {
                    // Class is not in the Bloom filter, hence the this `class` value does not
                    // contain the given class
                    false
                }
            }
            CaseSensitivity::AsciiCaseInsensitive => self.has_class_impl(name, case_sensitivity),
        }
    }
}

#[inline]
#[allow(clippy::arithmetic_side_effects)]
pub(crate) fn hash_class_name(name: &[u8]) -> u32 {
    let mut hasher = FxHasher::default();
    name.hash(&mut hasher);
    u32::try_from(hasher.finish() >> 32).expect("Invalid hash value")
}

/// A collection of HTML attributes.
#[derive(Debug)]
pub(crate) struct Attributes {
    /// Attribute names and their respective values.
    pub(crate) map: BTreeMap<QualName, StrTendril>,
    pub(crate) class: Option<Class>,
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
                class = Some(Class::new(attr.value));
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
