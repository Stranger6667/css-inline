mod attributes;
mod document;
mod element;
mod iter;
mod node;
mod parser;
mod selectors;
mod serializer;

pub(crate) use self::selectors::Specificity;
pub(crate) use document::Document;
pub(crate) use parser::InliningMode;
use smallvec::SmallVec;

/// Styles for a single element: (property name, specificity, value)
/// Uses `SmallVec` for cache-friendly linear search on small style counts.
pub(crate) type ElementStyleMap<'i> = SmallVec<[(&'i str, Specificity, &'i str); 4]>;

/// Maps node IDs to their accumulated styles.
/// Uses a Vec indexed by `NodeId` for O(1) access instead of hash lookups.
pub(crate) type DocumentStyleMap<'i> = Vec<Option<ElementStyleMap<'i>>>;
