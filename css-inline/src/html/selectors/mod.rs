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
    parser::{Component, ParseRelative, Selector as GenericSelector, SelectorParseErrorKind},
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

/// The anchor type for indexed selector lookup.
/// Determines which index to use for fast element lookup.
#[derive(Debug)]
pub(crate) enum SelectorAnchor<'a> {
    /// ID selector - O(1) lookup
    Id(&'a LocalName),
    /// Class selector - O(k) lookup where k = elements with this class
    Class(&'a LocalName),
    /// Tag selector - O(k) lookup where k = elements with this tag
    Tag(&'a LocalName),
    /// No usable anchor - must scan all elements
    None,
}

/// Extract the best anchor from a single selector.
/// Analyzes the rightmost compound selector (matched first) to find ID/class/tag.
#[inline]
fn selector_anchor(selector: &Selector) -> SelectorAnchor<'_> {
    // Iterate in match order (rightmost first)
    for component in selector.iter_raw_match_order() {
        match component {
            Component::ID(id) => {
                return SelectorAnchor::Id(id);
            }
            Component::Class(class) => {
                return SelectorAnchor::Class(class);
            }
            Component::LocalName(local_name) => {
                return SelectorAnchor::Tag(&local_name.name);
            }
            // Stop at combinators - we only analyze the rightmost compound
            Component::Combinator(_) => break,
            _ => {}
        }
    }
    SelectorAnchor::None
}

impl Selectors {
    /// Compile a list of selectors.
    #[inline]
    pub(super) fn compile(selectors: &str) -> Result<Selectors, ParseError<'_>> {
        parse(selectors).map(|list| Selectors(list.slice().into()))
    }

    /// Iterator over selectors.
    #[inline]
    pub(super) fn iter(&self) -> impl Iterator<Item = &Selector> {
        self.0.iter()
    }

    /// Get the best anchor for indexed lookup.
    /// If there are multiple selectors (comma-separated), returns None.
    #[inline]
    pub(crate) fn anchor(&self) -> SelectorAnchor<'_> {
        // For multiple selectors, we fall back to scanning all elements
        match self.0.as_slice() {
            [single] => selector_anchor(single),
            _ => SelectorAnchor::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    /// Assert that a selector compiles to the expected anchor type and value.
    macro_rules! assert_anchor {
        ($selector:expr, Id, $value:expr) => {
            let compiled = Selectors::compile($selector).unwrap();
            match compiled.anchor() {
                SelectorAnchor::Id(name) => assert_eq!(name.as_inner(), $value),
                other => panic!(
                    "Expected Id({:?}) for {:?}, got {:?}",
                    $value, $selector, other
                ),
            }
        };
        ($selector:expr, Class, $value:expr) => {
            let compiled = Selectors::compile($selector).unwrap();
            match compiled.anchor() {
                SelectorAnchor::Class(name) => assert_eq!(name.as_inner(), $value),
                other => panic!(
                    "Expected Class({:?}) for {:?}, got {:?}",
                    $value, $selector, other
                ),
            }
        };
        ($selector:expr, Tag, $value:expr) => {
            let compiled = Selectors::compile($selector).unwrap();
            match compiled.anchor() {
                SelectorAnchor::Tag(name) => assert_eq!(name.as_inner(), $value),
                other => panic!(
                    "Expected Tag({:?}) for {:?}, got {:?}",
                    $value, $selector, other
                ),
            }
        };
        ($selector:expr, None) => {
            let compiled = Selectors::compile($selector).unwrap();
            match compiled.anchor() {
                SelectorAnchor::None => {}
                other => panic!("Expected None for {:?}, got {:?}", $selector, other),
            }
        };
    }

    // ID selectors
    #[test_case("#myid", "myid"; "simple_id")]
    #[test_case("#my-id", "my-id"; "id_with_hyphen")]
    #[test_case("#myid.myclass", "myid"; "id_with_class")]
    #[test_case("div #myid", "myid"; "descendant_id")]
    fn test_id_anchor(selector: &str, expected: &str) {
        assert_anchor!(selector, Id, expected);
    }

    // Class selectors
    #[test_case(".myclass", "myclass"; "simple_class")]
    #[test_case(".my-class", "my-class"; "class_with_hyphen")]
    #[test_case(".my_class", "my_class"; "class_with_underscore")]
    #[test_case(".class1.class2", "class1"; "multiple_classes")]
    #[test_case("div > .myclass", "myclass"; "child_class")]
    #[test_case(".item:hover", "item"; "class_with_pseudo")]
    #[test_case(".btn[disabled]", "btn"; "class_with_attr")]
    #[test_case("*.myclass", "myclass"; "universal_with_class")]
    #[test_case(".item:not(.disabled)", "item"; "class_with_not")]
    fn test_class_anchor(selector: &str, expected: &str) {
        assert_anchor!(selector, Class, expected);
    }

    // Tag selectors
    #[test_case("div", "div"; "simple_div")]
    #[test_case("p", "p"; "simple_p")]
    #[test_case("span", "span"; "simple_span")]
    #[test_case("h1", "h1"; "simple_h1")]
    #[test_case("body", "body"; "simple_body")]
    #[test_case("div.myclass", "div"; "tag_with_class")]
    #[test_case("div#myid", "div"; "tag_with_id")]
    #[test_case("div.class1.class2", "div"; "tag_with_multiple_classes")]
    #[test_case("div p", "p"; "descendant")]
    #[test_case("div > p", "p"; "child")]
    #[test_case("h1 + p", "p"; "adjacent_sibling")]
    #[test_case("h1 ~ p", "p"; "general_sibling")]
    #[test_case("div ul li a.link", "a"; "deeply_nested")]
    #[test_case("a:hover", "a"; "tag_with_pseudo")]
    #[test_case("p:first-child", "p"; "tag_with_first_child")]
    #[test_case("li:nth-child(2)", "li"; "tag_with_nth_child")]
    #[test_case("a[href]", "a"; "tag_with_attr")]
    #[test_case("input[type=\"text\"]", "input"; "tag_with_attr_value")]
    #[test_case("nav.navbar ul.nav li.nav-item a.nav-link", "a"; "real_world")]
    #[test_case("div:not(.hidden)", "div"; "tag_with_not")]
    fn test_tag_anchor(selector: &str, expected: &str) {
        assert_anchor!(selector, Tag, expected);
    }

    // None (multiple selectors or universal)
    #[test_case("h1, h2, h3"; "multiple_tags")]
    #[test_case("div, .class, #id"; "multiple_mixed")]
    #[test_case("div, span"; "two_tags")]
    #[test_case("*"; "universal")]
    fn test_none_anchor(selector: &str) {
        assert_anchor!(selector, None);
    }
}
