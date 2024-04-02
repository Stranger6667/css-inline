use html5ever::{local_name, namespace_url, ns, tendril::StrTendril, QualName};
use rustc_hash::FxHasher;
use selectors::attr::CaseSensitivity;
use std::hash::{Hash, Hasher};

/// Class attribute value wrapper that provides a way to make fast class lookups.
#[derive(Debug)]
pub(crate) struct Class {
    pub(crate) value: StrTendril,
    pub(crate) cache: Cache,
}

static SELECTOR_WHITESPACE: &[char] = &[' ', '\t', '\n', '\r', '\x0C'];

/// This Bloom filter has 64 bits of storage and two hash functions which gives a decent false
/// positive rate given that most of the time an element has a very few number of classes.
#[derive(Debug, Copy, Clone)]
pub(crate) struct BloomFilter(u64);

impl BloomFilter {
    #[inline]
    fn new() -> BloomFilter {
        BloomFilter(0)
    }

    #[inline]
    fn insert_hash(&mut self, hash: u64) {
        self.0 |= 1 << (hash1(hash) % 64);
        self.0 |= 1 << (hash2(hash) % 64);
    }

    #[inline]
    fn might_contain_hash(self, hash: u64) -> bool {
        self.0 & (1 << (hash1(hash) % 64)) != 0 && self.0 & (1 << (hash2(hash) % 64)) != 0
    }

    /// Check whether this element MAY have the given class.
    /// If positive, then the class is probably there, and in this case we need to perform the actual check.
    /// Otherwise, the element does not have this class.
    #[inline]
    fn might_have_class(self, name: &[u8]) -> bool {
        let hash = hash_class_name(name);
        self.might_contain_hash(hash)
    }
}

const KEY_SIZE: usize = 32;
const KEY_MASK: u64 = u32::MAX as u64;

#[inline]
fn hash1(hash: u64) -> u64 {
    hash & KEY_MASK
}

#[inline]
fn hash2(hash: u64) -> u64 {
    (hash >> KEY_SIZE) & KEY_MASK
}

#[derive(Debug)]
pub(crate) enum Cache {
    /// In CSS selector matching, checking an element's class is frequent. Given that classes are
    /// often specific, most elements won't have the checked class. Leveraging this, we use a Bloom
    /// filter for a quick initial check. If positive, we do an actual check. This two-tier
    /// approach ensures fewer actual checks on class attributes.
    Bloom(BloomFilter),
    /// Element has a single class.
    Single,
}

impl Class {
    fn new(value: StrTendril) -> Class {
        // Build a Bloom filter for all element's classes
        let mut cache = BloomFilter::new();
        let mut classes = value.split(SELECTOR_WHITESPACE).filter(|s| !s.is_empty());
        if let Some(class) = classes.next() {
            // If the first class we split is the same as the input value, then we have just
            // a single class and a Bloom filter is not needed.
            if class.len() == value.len() {
                return Class {
                    value,
                    cache: Cache::Single,
                };
            }
            let hash = hash_class_name(class.as_bytes());
            cache.insert_hash(hash);
        }
        for class in classes {
            let hash = hash_class_name(class.as_bytes());
            cache.insert_hash(hash);
        }
        Class {
            value,
            cache: Cache::Bloom(cache),
        }
    }

    /// Manually check whether the class attribute value contains the given class.
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
        match (&self.cache, case_sensitivity) {
            (Cache::Single, case_sensitivity) => case_sensitivity.eq(self.value.as_bytes(), name),
            (Cache::Bloom(bloom_filter), CaseSensitivity::CaseSensitive) => {
                if bloom_filter.might_have_class(name) {
                    self.has_class_impl(name, case_sensitivity)
                } else {
                    // Class is not in the Bloom filter, hence this `class` value does not
                    // contain the given class
                    false
                }
            }
            (Cache::Bloom(_), CaseSensitivity::AsciiCaseInsensitive) => {
                self.has_class_impl(name, case_sensitivity)
            }
        }
    }
}

#[inline]
pub(crate) fn hash_class_name(name: &[u8]) -> u64 {
    let mut hasher = FxHasher::default();
    name.hash(&mut hasher);
    hasher.finish()
}

/// A collection of HTML attributes.
#[derive(Debug)]
pub(crate) struct Attributes {
    pub(crate) attributes: Vec<html5ever::Attribute>,
    /// The 'class' attribute value is separated for performance reasons.
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
    pub(crate) fn new(mut attributes: Vec<html5ever::Attribute>) -> Attributes {
        let mut class = None;
        if let Some(idx) = attributes
            .iter()
            .position(|attr| attr.name.local == local_name!("class"))
        {
            let attr = attributes.swap_remove(idx);
            class = Some(Class::new(attr.value));
        }
        Attributes { attributes, class }
    }

    pub(crate) fn find(&self, needle: &QualName) -> Option<&str> {
        self.attributes.iter().find_map(|probe| {
            if probe.name == *needle {
                Some(&*probe.value)
            } else {
                None
            }
        })
    }

    /// Checks if the attributes map contains a given local name.
    pub(crate) fn contains(&self, local: html5ever::LocalName) -> bool {
        self.get(local).is_some()
    }

    /// Get the value of the attribute with the given local name, if it exists.
    pub(crate) fn get(&self, local: html5ever::LocalName) -> Option<&str> {
        let needle = QualName::new(None, ns!(), local);
        self.find(&needle)
    }
}

#[cfg(test)]
mod tests {
    use super::Class;
    use selectors::attr::CaseSensitivity;
    use test_case::test_case;

    #[test_case("a b")]
    #[test_case("a")]
    fn test_has_class(value: &str) {
        let class = Class::new(value.into());
        assert!(class.has_class(b"a", CaseSensitivity::CaseSensitive));
        assert!(class.has_class(b"A", CaseSensitivity::AsciiCaseInsensitive));
        assert!(!class.has_class(b"c", CaseSensitivity::CaseSensitive));
        assert!(!class.has_class(b"C", CaseSensitivity::AsciiCaseInsensitive));
    }
}
