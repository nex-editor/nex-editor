//! Schema definition for document content validation.
//!
//! The schema defines:
//! - What node types are allowed
//! - What content each node type can contain
//! - What attributes each node type can have
//!
//! This is similar to ProseMirror's schema system.

use super::{ContentType, NodeContent, NodeType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Document schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    /// Block-level node definitions
    blocks: HashMap<NodeType, BlockSchema>,
    /// Inline node definitions
    inlines: HashMap<NodeType, InlineSchema>,
    /// Top-level content allowed
    top_level: ContentType,
    /// Marks/text styles
    marks: HashMap<&'static str, MarkSchema>,
}

/// Schema for block-level nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockSchema {
    /// Content type
    pub content: ContentType,
    /// Block-level attributes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attrs: Vec<BlockAttribute>,
}

/// Schema for inline nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineSchema {
    /// Inline-level attributes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attrs: Vec<InlineAttribute>,
}

/// Schema for text marks/styles
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkSchema {
    /// Whether this mark can be toggled
    #[serde(default)]
    pub toggleable: bool,
    /// Default value
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Exclude from this mark
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excludes: Vec<&'static str>,
}

/// Block-level attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockAttribute {
    /// ID attribute
    Id,
    /// Class attribute
    Class(String),
    /// Style attribute
    Style(String),
    /// Custom attribute
    Custom {
        name: String,
        #[serde(default)]
        required: bool,
    },
}

/// Inline attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InlineAttribute {
    /// Link URL
    Href,
    /// Title
    Title,
    /// Class
    Class(String),
    /// Style
    Style(String),
    /// Target for links
    Target,
    /// Custom attribute
    Custom {
        name: String,
        #[serde(default)]
        required: bool,
    },
}

/// Validation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult<'a> {
    valid: bool,
    node: Option<&'a NodeType>,
    message: Option<String>,
}

impl<'a> ValidationResult<'a> {
    pub fn ok() -> Self {
        Self {
            valid: true,
            node: None,
            message: None,
        }
    }

    pub fn error(node: &'a NodeType, message: &str) -> Self {
        Self {
            valid: false,
            node: Some(node),
            message: Some(message.to_string()),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

/// Schema for the default document type
pub const DEFAULT_SCHEMA: Schema = Schema::new();

impl Schema {
    /// Create the default schema (similar to ProseMirror's doc schema)
    pub const fn new() -> Self {
        let mut blocks = HashMap::new();
        let mut inlines = HashMap::new();
        let mut marks = HashMap::new();

        // Document
        blocks.insert(
            NodeType::Document,
            BlockSchema {
                content: ContentType::Block(vec![
                    NodeType::Paragraph,
                    NodeType::Heading { level: 1 },
                    NodeType::Heading { level: 2 },
                    NodeType::Heading { level: 3 },
                    NodeType::Heading { level: 4 },
                    NodeType::Heading { level: 5 },
                    NodeType::Heading { level: 6 },
                    NodeType::List { ordered: false },
                    NodeType::List { ordered: true },
                    NodeType::Blockquote,
                    NodeType::CodeBlock { lang: None },
                    NodeType::Divider,
                ]),
                attrs: Vec::new(),
            },
        );

        // Paragraph
        blocks.insert(
            NodeType::Paragraph,
            BlockSchema {
                content: ContentType::Inline,
                attrs: Vec::new(),
            },
        );

        // Headings
        blocks.insert(
            NodeType::Heading { level: 1 },
            BlockSchema {
                content: ContentType::Inline,
                attrs: Vec::new(),
            },
        );

        // Lists
        blocks.insert(
            NodeType::List { ordered: false },
            BlockSchema {
                content: ContentType::Block(vec![NodeType::ListItem]),
                attrs: Vec::new(),
            },
        );

        blocks.insert(
            NodeType::List { ordered: true },
            BlockSchema {
                content: ContentType::Block(vec![NodeType::ListItem]),
                attrs: Vec::new(),
            },
        );

        // List item
        blocks.insert(
            NodeType::ListItem,
            BlockSchema {
                content: ContentType::Block(vec![NodeType::Paragraph]),
                attrs: Vec::new(),
            },
        );

        // Blockquote
        blocks.insert(
            NodeType::Blockquote,
            BlockSchema {
                content: ContentType::Block(vec![NodeType::Paragraph]),
                attrs: Vec::new(),
            },
        );

        // Code block
        blocks.insert(
            NodeType::CodeBlock { lang: None },
            BlockSchema {
                content: ContentType::Text,
                attrs: vec![BlockAttribute::Custom {
                    name: "language".to_string(),
                    required: false,
                }],
            },
        );

        // Divider
        blocks.insert(
            NodeType::Divider,
            BlockSchema {
                content: ContentType::Empty,
                attrs: Vec::new(),
            },
        );

        // Text (inline)
        inlines.insert(
            NodeType::Text,
            InlineSchema {
                attrs: Vec::new(),
            },
        );

        // Link
        inlines.insert(
            NodeType::Link { url: String::new(), title: None },
            InlineSchema {
                attrs: vec![
                    InlineAttribute::Href,
                    InlineAttribute::Title,
                    InlineAttribute::Target,
                ],
            },
        );

        // Image
        inlines.insert(
            NodeType::Image { src: String::new(), alt: None },
            InlineSchema {
                attrs: vec![
                    InlineAttribute::Custom {
                        name: "src".to_string(),
                        required: true,
                    },
                    InlineAttribute::Custom {
                        name: "alt".to_string(),
                        required: false,
                    },
                ],
            },
        );

        // Marks
        marks.insert(
            "strong",
            MarkSchema {
                toggleable: true,
                default: None,
                excludes: vec!["strong", "em"],
            },
        );

        marks.insert(
            "em",
            MarkSchema {
                toggleable: true,
                default: None,
                excludes: vec!["strong", "em"],
            },
        );

        marks.insert(
            "code",
            MarkSchema {
                toggleable: true,
                default: None,
                excludes: Vec::new(),
            },
        );

        marks.insert(
            "link",
            MarkSchema {
                toggleable: false,
                default: None,
                excludes: Vec::new(),
            },
        );

        Self {
            blocks,
            inlines,
            top_level: ContentType::Block(vec![
                NodeType::Paragraph,
                NodeType::Heading { level: 1 },
            ]),
            marks,
        }
    }

    /// Check if a node type is valid
    pub fn is_valid_node_type(&self, node_type: &NodeType) -> bool {
        self.blocks.contains_key(node_type)
            || self.inlines.contains_key(node_type)
            || matches!(node_type, NodeType::Document)
    }

    /// Validate a block node
    pub fn validate_block(&self, node_type: &NodeType, content: &NodeContent) -> ValidationResult {
        if let Some(schema) = self.blocks.get(node_type) {
            self.validate_content(&schema.content, content)
        } else {
            ValidationResult::error(node_type, "Unknown block type")
        }
    }

    /// Validate content against a content type
    fn validate_content(&self, allowed: &ContentType, content: &NodeContent) -> ValidationResult {
        match allowed {
            ContentType::Empty => {
                if matches!(content, NodeContent::Empty) {
                    ValidationResult::ok()
                } else {
                    ValidationResult::error(
                        &NodeType::Paragraph,
                        "Content not allowed here",
                    )
                }
            }
            ContentType::Inline => {
                if matches!(content, NodeContent::Text(_)) {
                    ValidationResult::ok()
                } else {
                    ValidationResult::error(
                        &NodeType::Text,
                        "Inline content required",
                    )
                }
            }
            ContentType::Block(allowed_types) => {
                if let NodeContent::Mixed(mixed) = content {
                    for child_id in &mixed.children {
                        // Validate each child type
                        // This would need the actual node to check
                    }
                    ValidationResult::ok()
                } else {
                    ValidationResult::error(
                        &NodeType::Paragraph,
                        "Block content required",
                    )
                }
            }
            ContentType::Mixed { .. } => ValidationResult::ok(),
        }
    }

    /// Get the schema for a block type
    pub fn get_block_schema(&self, node_type: &NodeType) -> Option<&BlockSchema> {
        self.blocks.get(node_type)
    }

    /// Get the schema for an inline type
    pub fn get_inline_schema(&self, node_type: &NodeType) -> Option<&InlineSchema> {
        self.inlines.get(node_type)
    }

    /// Check if a mark is defined
    pub fn has_mark(&self, name: &str) -> bool {
        self.marks.contains_key(name)
    }

    /// Get mark schema
    pub fn get_mark(&self, name: &str) -> Option<&MarkSchema> {
        self.marks.get(name)
    }
}

impl Default for Schema {
    fn default() -> Self {
        DEFAULT_SCHEMA
    }
}
