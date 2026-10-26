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

use super::document::NameSet;
use selectors::{
    parser::{
        Combinator, Component, ParseRelative, Selector as GenericSelector, SelectorParseErrorKind,
    },
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

pub(crate) type ParseError = cssparser::ParseError<SelectorParseErrorKind>;

/// Parse CSS selectors into `SelectorList`.
fn parse(selectors: &str) -> Result<SelectorList<InlinerSelectors>, ParseError> {
    let parser = &mut cssparser::Parser::new(selectors);
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
    /// Attribute selector - O(k) lookup where k = elements with this attribute
    Attribute(&'a LocalName),
    /// `:root` - only the top-level elements
    Root,
    /// No usable anchor - must scan all elements
    None,
}

/// Extract the best anchor from a single selector.
/// Analyzes the rightmost compound selector (matched first) to find `:root` or ID/class/tag,
/// falling back to an attribute name.
#[inline]
fn selector_anchor(selector: &Selector) -> SelectorAnchor<'_> {
    let mut anchor = None;
    let mut attribute = None;
    // Iterate in match order (rightmost first)
    for component in selector.iter_raw_match_order() {
        match component {
            // The most selective anchor, wherever it is in the compound
            Component::Root => return SelectorAnchor::Root,
            Component::ID(id) => {
                anchor.get_or_insert(SelectorAnchor::Id(id));
            }
            Component::Class(class) => {
                anchor.get_or_insert(SelectorAnchor::Class(class));
            }
            // Mixed-case names match differently on HTML and foreign elements, so only
            // lowercase ones are used
            Component::LocalName(local_name) if local_name.name == local_name.lower_name => {
                anchor.get_or_insert(SelectorAnchor::Tag(&local_name.name));
            }
            Component::AttributeInNoNamespace { local_name, .. } => {
                attribute.get_or_insert(local_name);
            }
            Component::AttributeInNoNamespaceExists {
                local_name,
                local_name_lower,
            } if local_name == local_name_lower => {
                attribute.get_or_insert(local_name);
            }
            // Stop at combinators - we only analyze the rightmost compound
            Component::Combinator(_) => break,
            _ => {}
        }
    }
    anchor.unwrap_or_else(|| attribute.map_or(SelectorAnchor::None, SelectorAnchor::Attribute))
}

/// Names that some ancestor element of every element matching the selector has.
/// Only names outside of nested selectors (`:not()`, ...) count.
pub(crate) fn ancestor_names(selector: &Selector) -> NameSet {
    let mut names = NameSet::default();
    // A compound left of ` ` or `>` matches an ancestor. One left of `+` or `~` matches a
    // sibling and is skipped; siblings share the parent, so compounds further left of
    // ` ` or `>` still match ancestors.
    let mut on_ancestor = false;
    for component in selector.iter_raw_match_order() {
        match component {
            Component::Combinator(Combinator::Descendant | Combinator::Child) => {
                on_ancestor = true;
            }
            Component::Combinator(Combinator::NextSibling | Combinator::LaterSibling) => {
                on_ancestor = false;
            }
            Component::Combinator(_) => return NameSet::default(),
            // Matching runs in no-quirks mode, where IDs and classes are case-sensitive
            Component::ID(id) if on_ancestor => names.insert_id(id.as_bytes()),
            Component::Class(class) if on_ancestor => names.insert_class(class.as_bytes()),
            // HTML elements match `lower_name`, foreign ones match `name`; both lowercase to
            // `lower_name`, and ancestor tags are stored lowercased
            Component::LocalName(local_name) if on_ancestor => {
                names.insert_tag(local_name.lower_name.as_bytes());
            }
            _ => {}
        }
    }
    names
}

impl Selectors {
    /// Compile a list of selectors.
    #[inline]
    pub(super) fn compile(selectors: &str) -> Result<Selectors, ParseError> {
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
        ($selector:expr, Attribute, $value:expr) => {
            let compiled = Selectors::compile($selector).unwrap();
            match compiled.anchor() {
                SelectorAnchor::Attribute(name) => assert_eq!(name.as_inner(), $value),
                other => panic!(
                    "Expected Attribute({:?}) for {:?}, got {:?}",
                    $value, $selector, other
                ),
            }
        };
        ($selector:expr, Root) => {
            let compiled = Selectors::compile($selector).unwrap();
            match compiled.anchor() {
                SelectorAnchor::Root => {}
                other => panic!("Expected Root for {:?}, got {:?}", $selector, other),
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
    #[test_case("DIV.item", "item"; "uppercase_tag_with_class")]
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

    // Attribute selectors
    #[test_case("[hidden]", "hidden"; "attribute_exists")]
    #[test_case("[type=button]", "type"; "attribute_value")]
    #[test_case("[data-role~=\"tab\"]", "data-role"; "attribute_includes")]
    #[test_case("[data-a][data-b]", "data-a"; "multiple_attributes")]
    #[test_case("[data-a]:hover", "data-a"; "attribute_with_pseudo")]
    #[test_case(".nav [data-x]", "data-x"; "descendant_attribute")]
    fn test_attribute_anchor(selector: &str, expected: &str) {
        assert_anchor!(selector, Attribute, expected);
    }

    // ID, class and tag take precedence over attributes
    #[test_case("[data-x]#myid", "myid"; "attribute_with_id")]
    fn test_attribute_with_id_anchor(selector: &str, expected: &str) {
        assert_anchor!(selector, Id, expected);
    }

    #[test_case("[data-x].item", "item"; "attribute_with_class")]
    fn test_attribute_with_class_anchor(selector: &str, expected: &str) {
        assert_anchor!(selector, Class, expected);
    }

    #[test_case(":root"; "root")]
    #[test_case("html:root"; "tag_root")]
    #[test_case(":root.dark"; "root_class")]
    #[test_case("#x:root"; "id_root")]
    #[test_case("[data-x]:root"; "attribute_root")]
    #[test_case("body :root"; "descendant_root")]
    fn test_root_anchor(selector: &str) {
        assert_anchor!(selector, Root);
    }

    #[test_case(":root > body", "body"; "root_child")]
    #[test_case(":root p", "p"; "root_descendant")]
    fn test_root_not_rightmost_anchor(selector: &str, expected: &str) {
        assert_anchor!(selector, Tag, expected);
    }

    // None (multiple selectors or universal)
    #[test_case("h1, h2, h3"; "multiple_tags")]
    #[test_case("div, .class, #id"; "multiple_mixed")]
    #[test_case("div, span"; "two_tags")]
    #[test_case("*"; "universal")]
    #[test_case("[viewBox]"; "mixed_case_attribute")]
    #[test_case("H1"; "uppercase_tag")]
    #[test_case("svg foreignObject"; "mixed_case_tag")]
    #[test_case("[data-x] *"; "attribute_in_ancestor")]
    #[test_case(":not(:root)"; "not_root")]
    #[test_case(":root, p"; "root_in_list")]
    fn test_none_anchor(selector: &str) {
        assert_anchor!(selector, None);
    }
}
