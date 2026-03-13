//! Schema definition and validation
//!
//! Defines document schemas that specify which node types are allowed
//! and their content expressions (content constraints).

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Errors that can occur during schema validation
#[derive(Debug, Error, PartialEq)]
pub enum SchemaError {
    #[error("Unknown node type: {0}")]
    UnknownNodeType(String),

    #[error("Invalid content for node type {node_type}: {reason}")]
    InvalidContent {
        node_type: String,
        reason: String,
    },

    #[error("Node {node_type} requires content but was empty")]
    MissingContent { node_type: String },

    #[error("Node {node_type} allows no content but found {found:?}")]
    UnexpectedContent { node_type: String, found: String },
}

/// Content expression pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentExpr {
    /// Required content (e.g., "block+")
    Required(String),
    /// Optional content (e.g., "block*")
    Optional(String),
    /// Sequence of required followed by optional
    Sequence(String, String),
    /// Choice between patterns
    Choice(Vec<ContentExpr>),
    /// No content allowed
    Empty,
    /// Any content
    Any,
}

/// Helper for matching content against schema rules
#[derive(Debug, Clone)]
pub struct ContentMatcher {
    /// The content expression pattern
    expr: ContentExpr,
    /// Map of node type names to their categories
    category_map: HashMap<String, String>,
}

impl ContentMatcher {
    /// Create a new content matcher
    pub fn new(expr: ContentExpr, category_map: HashMap<String, String>) -> Self {
        Self { expr, category_map }
    }

    /// Check if a sequence of nodes matches this pattern
    pub fn matches(&self, nodes: &[super::Node]) -> Result<(), SchemaError> {
        self.matches_with_expr(&self.expr, nodes, 0, nodes.len())
    }

    fn matches_with_expr(
        &self,
        expr: &ContentExpr,
        nodes: &[super::Node],
        start: usize,
        end: usize,
    ) -> Result<(), SchemaError> {
        let node_slice = &nodes[start..end];
        match expr {
            ContentExpr::Required(pattern) => {
                if node_slice.is_empty() {
                    Err(SchemaError::MissingContent { node_type: pattern.to_string() })
                } else if !self.pattern_matches(pattern, node_slice) {
                    Err(SchemaError::InvalidContent {
                        node_type: pattern.to_string(),
                        reason: format!("Required content '{}' not matched", pattern),
                    })
                } else {
                    Ok(())
                }
            }
            ContentExpr::Optional(pattern) => {
                if !node_slice.is_empty() && !self.pattern_matches(pattern, node_slice) {
                    Err(SchemaError::InvalidContent {
                        node_type: pattern.to_string(),
                        reason: format!("Optional content '{}' not matched", pattern),
                    })
                } else {
                    Ok(())
                }
            }
            ContentExpr::Sequence(req, opt) => {
                // Check required part
                if node_slice.is_empty() {
                    return Err(SchemaError::MissingContent { node_type: req.to_string() });
                }
                let req_count = self.count_matching(req, node_slice);
                if req_count == 0 {
                    return Err(SchemaError::InvalidContent {
                        node_type: req.to_string(),
                        reason: format!("Required content '{}' not found", req),
                    });
                }
                // Check optional part
                let opt_count = self.count_matching(opt, node_slice);
                if req_count + opt_count != node_slice.len() {
                    Err(SchemaError::InvalidContent {
                        node_type: format!("{}+{}?", req, opt),
                        reason: "Unexpected content after pattern".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            ContentExpr::Choice(choices) => {
                for choice in choices {
                    if self.matches_with_expr(choice, nodes, start, end).is_ok() {
                        return Ok(());
                    }
                }
                Err(SchemaError::InvalidContent {
                    node_type: "choice".to_string(),
                    reason: "No choice matched".to_string(),
                })
            }
            ContentExpr::Empty => {
                if !node_slice.is_empty() {
                    Err(SchemaError::UnexpectedContent {
                        node_type: "empty".to_string(),
                        found: format!("{:?}", node_slice.iter().map(|n| n.type_name()).collect::<Vec<_>>()),
                    })
                } else {
                    Ok(())
                }
            }
            ContentExpr::Any => Ok(()),
        }
    }

    fn pattern_matches(&self, pattern: &str, nodes: &[super::Node]) -> bool {
        if pattern.ends_with('+') {
            // Required one or more
            let inner = &pattern[..pattern.len() - 1];
            let count = self.count_matching(inner, nodes);
            count > 0 && count == nodes.len()
        } else if pattern.ends_with('*') {
            // Optional zero or more
            let inner = &pattern[..pattern.len() - 1];
            let count = self.count_matching(inner, nodes);
            count == nodes.len()
        } else {
            // Exact match
            nodes.len() == 1 && self.type_matches(pattern, &nodes[0])
        }
    }

    fn count_matching(&self, pattern: &str, nodes: &[super::Node]) -> usize {
        let (prefix, is_plus, is_star) = if let Some(p) = pattern.strip_suffix('+') {
            (p, true, false)
        } else if let Some(p) = pattern.strip_suffix('*') {
            (p, false, true)
        } else {
            (pattern, false, false)
        };

        let mut count = 0;
        for node in nodes {
            if self.type_matches(prefix, node) {
                count += 1;
            } else if !is_star && !is_plus {
                break;
            }
        }

        // For plus, must be at least one
        if is_plus && count == 0 {
            0
        } else {
            count
        }
    }

    fn type_matches(&self, pattern: &str, node: &super::Node) -> bool {
        if pattern == "block" {
            return node.is_block();
        }
        if pattern == "text" {
            return node.is_text();
        }
        // Check exact type match
        if pattern == node.type_name() {
            return true;
        }
        // Check category match
        if let Some(category) = self.category_map.get(pattern) {
            // Check if node's type is in this category
            return category == node.type_name() ||
                   self.category_map.get(node.type_name()) == Some(category);
        }
        false
    }
}

/// Specification for a schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaSpec {
    /// Top-level node type
    pub top_node: String,
    /// Map of node type names to their content expressions
    pub nodes: HashMap<String, ContentExpr>,
    /// Map of mark type names to attributes
    pub marks: HashMap<String, serde_json::Value>,
    /// Default inline node type
    pub default_inline: Option<String>,
    /// Default block node type
    pub default_block: Option<String>,
}

impl Default for SchemaSpec {
    fn default() -> Self {
        let mut nodes = HashMap::new();
        nodes.insert("doc".to_string(), ContentExpr::Optional("block*".to_string()));
        nodes.insert("paragraph".to_string(), ContentExpr::Optional("text*".to_string()));
        nodes.insert("heading".to_string(), ContentExpr::Optional("text*".to_string()));
        nodes.insert("blockquote".to_string(), ContentExpr::Optional("block*".to_string()));
        nodes.insert("horizontal_rule".to_string(), ContentExpr::Empty);
        nodes.insert("text".to_string(), ContentExpr::Any);

        let mut marks = HashMap::new();
        marks.insert("bold".to_string(), serde_json::json!({}));
        marks.insert("italic".to_string(), serde_json::json!({}));
        marks.insert("link".to_string(), serde_json::json!({"href": ""}));

        Self {
            top_node: "doc".to_string(),
            nodes,
            marks,
            default_inline: Some("text".to_string()),
            default_block: Some("paragraph".to_string()),
        }
    }
}

/// Schema definition
///
/// A schema defines what nodes are valid in a document and what
/// content constraints each node type has.
#[derive(Debug, Clone)]
pub struct Schema {
    /// Schema specification
    spec: SchemaSpec,
    /// Cache of content matchers for each node type
    matchers: HashMap<String, ContentMatcher>,
}

impl serde::Serialize for Schema {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Schema", 1)?;
        s.serialize_field("spec", &self.spec)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for Schema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Spec,
        }

        struct SchemaVisitor;

        impl<'de> serde::de::Visitor<'de> for SchemaVisitor {
            type Value = Schema;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Schema struct")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Schema, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut spec = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Spec => spec = Some(map.next_value()?),
                    }
                }

                let spec = spec.unwrap_or_default();
                Ok(Schema::new(spec))
            }
        }

        const FIELDS: &[&str] = &["spec"];
        deserializer.deserialize_struct("Schema", FIELDS, SchemaVisitor)
    }
}

impl Schema {
    /// Create a new schema from a specification
    pub fn new(spec: SchemaSpec) -> Self {
        let mut matchers = HashMap::new();
        for (name, expr) in &spec.nodes {
            // Build category map (same as node type name = its own category)
            let mut category_map = HashMap::new();
            for node_name in spec.nodes.keys() {
                category_map.insert(node_name.clone(), node_name.clone());
            }
            // Add mark categories
            for mark_name in spec.marks.keys() {
                category_map.insert(mark_name.clone(), mark_name.clone());
            }

            matchers.insert(name.clone(), ContentMatcher::new(expr.clone(), category_map));
        }

        Self { spec, matchers }
    }

    /// Create a schema with default specification
    pub fn default() -> Self {
        Self::new(SchemaSpec::default())
    }

    /// Get the top node type name
    pub fn top_node_name(&self) -> &str {
        &self.spec.top_node
    }

    /// Check if a node type is valid
    pub fn is_valid_node_type(&self, type_name: &str) -> bool {
        self.spec.nodes.contains_key(type_name)
    }

    /// Check if a mark type is valid
    pub fn is_valid_mark_type(&self, type_name: &str) -> bool {
        self.spec.marks.contains_key(type_name)
    }

    /// Validate a node against the schema
    pub fn validate_node(&self, node: &super::Node) -> Result<(), SchemaError> {
        let type_name = node.type_name();

        // Check if node type is known
        if !self.is_valid_node_type(type_name) {
            return Err(SchemaError::UnknownNodeType(type_name.to_string()));
        }

        // Get the content expression for this node type
        let matcher = self.matchers.get(type_name)
            .ok_or_else(|| SchemaError::UnknownNodeType(type_name.to_string()))?;

        // Validate content
        let content = match &node.content {
            super::NodeContent::Nodes(nodes) => nodes.as_slice(),
            super::NodeContent::Text(_) => &[],
        };

        matcher.matches(content)
    }

    /// Validate child nodes for a parent node
    pub fn validate_children(&self, parent_type: &str, children: &[super::Node]) -> Result<(), SchemaError> {
        let matcher = self.matchers.get(parent_type)
            .ok_or_else(|| SchemaError::UnknownNodeType(parent_type.to_string()))?;

        matcher.matches(children)
    }

    /// Get the default block type
    pub fn default_block_type(&self) -> Option<&str> {
        self.spec.default_block.as_deref()
    }

    /// Get the default inline type
    pub fn default_inline_type(&self) -> Option<&str> {
        self.spec.default_inline.as_deref()
    }
}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        self.spec == other.spec
    }
}

impl Eq for Schema {}

impl Default for Schema {
    fn default() -> Self {
        Self::default()
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Schema(top: {})", self.top_node_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::Node;

    #[test]
    fn test_schema_default() {
        let schema = Schema::default();

        assert_eq!(schema.top_node_name(), "doc");
        assert!(schema.is_valid_node_type("paragraph"));
        assert!(schema.is_valid_node_type("text"));
        assert!(!schema.is_valid_node_type("unknown"));
    }

    #[test]
    fn test_schema_validate_node() {
        let schema = Schema::default();

        // Valid paragraph with text
        let para = Node::new_block("paragraph", vec![Node::new_text("Hello")]);
        assert!(schema.validate_node(&para).is_ok());

        // Valid paragraph without text (optional)
        let para = Node::new_block("paragraph", vec![]);
        assert!(schema.validate_node(&para).is_ok());

        // Invalid: hr with content
        let hr = Node::new_block("horizontal_rule", vec![Node::new_text("x")]);
        assert!(schema.validate_node(&hr).is_err());
    }

    #[test]
    fn test_schema_validate_children() {
        let schema = Schema::default();

        // Valid children for doc
        let children = vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
            Node::new_block("paragraph", vec![Node::new_text("World")]),
        ];
        assert!(schema.validate_children("doc", &children).is_ok());

        // Invalid: text as direct child of doc
        let children = vec![Node::new_text("Hello")];
        assert!(schema.validate_children("doc", &children).is_err());
    }

    #[test]
    fn test_content_matcher_required() {
        let spec = SchemaSpec {
            top_node: "doc".to_string(),
            nodes: [("container".to_string(), ContentExpr::Required("item+".to_string()))]
                .into_iter()
                .collect(),
            marks: HashMap::new(),
            default_inline: None,
            default_block: None,
        };
        let schema = Schema::new(spec);

        let item = Node::new_block("item", vec![]);

        // Required: at least one
        let children = vec![item.clone()];
        assert!(schema.validate_children("container", &children).is_ok());

        // Required: multiple
        let children = vec![item.clone(), item.clone()];
        assert!(schema.validate_children("container", &children).is_ok());

        // Required: zero (should fail)
        let children: Vec<Node> = vec![];
        assert!(schema.validate_children("container", &children).is_err());
    }

    #[test]
    fn test_content_matcher_optional() {
        let spec = SchemaSpec {
            top_node: "doc".to_string(),
            nodes: [("container".to_string(), ContentExpr::Optional("item*".to_string()))]
                .into_iter()
                .collect(),
            marks: HashMap::new(),
            default_inline: None,
            default_block: None,
        };
        let schema = Schema::new(spec);

        let item = Node::new_block("item", vec![]);

        // Optional: zero
        let children: Vec<Node> = vec![];
        assert!(schema.validate_children("container", &children).is_ok());

        // Optional: one
        let children = vec![item.clone()];
        assert!(schema.validate_children("container", &children).is_ok());

        // Optional: multiple
        let children = vec![item.clone(), item.clone()];
        assert!(schema.validate_children("container", &children).is_ok());
    }
}
