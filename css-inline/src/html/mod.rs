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
pub use document::DEFAULT_HTML_TREE_CAPACITY;
pub(crate) use node::NodeId;
