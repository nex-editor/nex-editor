//! Model layer - the foundation of the editor
//!
//! Contains core data structures: Node, Schema, and ResolvedPos

pub mod node;
pub mod schema;
pub mod resolved_pos;

pub use node::{Node, NodeContent, ContentSize};
pub use schema::{Schema, SchemaSpec, ContentExpr, ContentMatcher};
pub use resolved_pos::{ResolvedPos, Descendant};
