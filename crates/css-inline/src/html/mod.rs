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
pub(crate) use node::{NodeData, NodeId};
pub(crate) use parser::InliningMode;
use smallvec::SmallVec;

/// Cascade precedence of a declaration: layer rank, then selector specificity.
/// When equal, a rule later in source order takes precedence.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Precedence(pub(crate) u32, pub(crate) Specificity);

/// Styles for a single element: (property name, precedence, value)
pub(crate) type ElementStyleMap<'i> = SmallVec<[(&'i str, Precedence, &'i str); 4]>;

/// Maps node IDs to their accumulated styles.
pub(crate) type DocumentStyleMap<'i> = Vec<Option<ElementStyleMap<'i>>>;
