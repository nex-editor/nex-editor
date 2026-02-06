//! Node types and structures for the document tree.
//!
//! This module defines:
//! - `NodeId`: Unique identifier for nodes
//! - `NodeType`: Types of nodes (block, inline, text)
//! - `NodeContent`: Content stored in nodes
//! - `Node`: The complete node structure

use loro::LoroText;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub struct NodeId(pub u64);

impl NodeId {
    /// Create a new node ID
    pub fn new(n: u64) -> Self {
        Self(n)
    }

    /// Get the inner value
    pub fn inner(&self) -> u64 {
        self.0
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s.parse().unwrap_or(0))
    }
}

impl From<NodeId> for String {
    fn from(id: NodeId) -> Self {
        id.0.to_string()
    }
}

/// Types of nodes in the document tree
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    // === 容器节点 ===
    /// The root document node
    Document,

    // === 块级节点 ===
    /// Paragraph (default block)
    Paragraph,
    /// Heading with level (h1-h6)
    Heading {
        #[serde(default = "default_level_1")]
        level: u8,
    },
    /// List with ordering
    List {
        #[serde(default)]
        ordered: bool,
    },
    /// List item (must be child of List)
    ListItem,
    /// Block quote
    Blockquote,
    /// Code block with optional language
    CodeBlock {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lang: Option<String>,
    },
    /// Horizontal divider
    Divider,
    /// Custom container
    CustomContainer(&'static str),

    // === 内联节点 ===
    /// Plain text (leaf node)
    Text,
    /// Link with URL
    Link {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    /// Image with source and optional alt
    Image {
        src: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
    },
    /// Mention or tag
    Mention {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    /// Emoji or inline icon
    Emoji {
        char: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// Custom inline
    CustomInline(&'static str),

    // === 特殊节点 ===
    /// Table structure
    Table,
    /// Table row
    TableRow,
    /// Table cell
    TableCell,
    /// Table header cell
    TableHeaderCell,
    /// Collapsed/folded container
    Foldable(&'static str),
}

fn default_level_1() -> u8 {
    1
}

impl NodeType {
    /// Check if this is a block-level node
    pub fn is_block(&self) -> bool {
        matches!(
            self,
            NodeType::Paragraph
                | NodeType::Heading { .. }
                | NodeType::List { .. }
                | NodeType::ListItem
                | NodeType::Blockquote
                | NodeType::CodeBlock { .. }
                | NodeType::Divider
                | NodeType::Table
                | NodeType::TableRow
                | NodeType::TableCell
                | NodeType::TableHeaderCell
        )
    }

    /// Check if this is an inline node
    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            NodeType::Text
                | NodeType::Link { .. }
                | NodeType::Image { .. }
                | NodeType::Mention { .. }
                | NodeType::Emoji { .. }
        )
    }

    /// Check if this is a leaf node (no children)
    pub fn is_leaf(&self) -> bool {
        matches!(
            self,
            NodeType::Text
                | NodeType::Image { .. }
                | NodeType::Mention { .. }
                | NodeType::Emoji { .. }
        )
    }

    /// Get the default content type for this node
    pub fn default_content(&self) -> NodeContent {
        match self {
            NodeType::Document => NodeContent::Empty,
            NodeType::Paragraph | NodeType::Heading { .. } | NodeType::Blockquote | NodeType::ListItem => {
                NodeContent::Mixed(MixedContent::text())
            }
            NodeType::List { .. } => NodeContent::Mixed(MixedContent::block(vec![NodeType::ListItem])),
            NodeType::CodeBlock { lang } => NodeContent::Text(TextContent::new()),
            NodeType::Divider => NodeContent::Empty,
            NodeType::Table => NodeContent::Mixed(MixedContent::block(vec![NodeType::TableRow])),
            NodeType::TableRow => NodeContent::Mixed(MixedContent::block(vec![
                NodeType::TableCell,
                NodeType::TableHeaderCell,
            ])),
            NodeType::TableCell | NodeType::TableHeaderCell => {
                NodeContent::Mixed(MixedContent::text())
            }
            NodeType::Text => NodeContent::Text(TextContent::new()),
            NodeType::Link { .. } | NodeType::Image { .. } | NodeType::Mention { .. } | NodeType::Emoji { .. } => {
                NodeContent::Empty
            }
            NodeType::CustomContainer(_) | NodeType::CustomInline(_) | NodeType::Foldable(_) => {
                NodeContent::Empty
            }
        }
    }
}

/// Content stored in a node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeContent {
    /// Empty content (for container nodes)
    Empty,

    /// Text content with styles
    Text(TextContent),

    /// Mixed content (block with inline and/or block children)
    Mixed(MixedContent),

    /// Atomic content (cannot be split)
    Atom {
        kind: AtomKind,
        data: serde_json::Value,
    },
}

/// Text content with rich styling support
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextContent {
    /// The underlying LoroText container
    #[serde(skip)]
    pub text: LoroText,
    /// Default text styles
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_styles: Vec<TextStyle>,
}

impl TextContent {
    /// Create new empty text content
    pub fn new() -> Self {
        Self {
            text: LoroText::new(),
            default_styles: Vec::new(),
        }
    }

    /// Create with initial text
    pub fn with_text(text: &str) -> Self {
        let t = LoroText::new();
        let _ = t.insert(0, text);
        Self {
            text: t,
            default_styles: Vec::new(),
        }
    }

    /// Get text length in unicode chars
    pub fn len(&self) -> usize {
        self.text.len_unicode()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// Text style mark
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextStyle {
    /// Style key (e.g., "bold", "italic", "color")
    pub key: String,
    /// Style value
    pub value: serde_json::Value,
    /// Whether this is an expansion mark
    #[serde(default)]
    pub expand: bool,
}

/// Kinds of atomic content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AtomKind {
    /// Inline equation (KaTeX, MathML)
    MathInline,
    /// Block equation
    MathBlock,
    /// Code snippet with syntax highlighting
    Code,
    /// Media embed
    Media,
    /// Custom atom
    Custom(&'static str),
}

/// Mixed content with children
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MixedContent {
    /// Child node IDs in order
    #[serde(default)]
    pub children: Vec<NodeId>,
    /// Schema for allowed content
    #[serde(default)]
    pub content: ContentType,
}

impl MixedContent {
    /// Create for text-only content
    pub fn text() -> Self {
        Self {
            children: Vec::new(),
            content: ContentType::Inline,
        }
    }

    /// Create for block-only content
    pub fn block(allowed: Vec<NodeType>) -> Self {
        Self {
            children: Vec::new(),
            content: ContentType::Block(allowed),
        }
    }
}

/// Allowed content type in a node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// No content allowed
    Empty,
    /// Inline content only (text, links, etc.)
    Inline,
    /// Block content only
    Block(Vec<NodeType>),
    /// Mixed inline and block content
    Mixed {
        /// Allowed inline types
        inline: Vec<NodeType>,
        /// Allowed block types
        blocks: Vec<NodeType>,
    },
}

/// Node metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeMeta {
    /// Whether this node is collapsed/folded
    #[serde(default)]
    pub collapsed: bool,
    /// Whether this node is deleted (for undo/redo)
    #[serde(default)]
    pub deleted: bool,
    /// Custom attributes
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub attrs: HashMap<String, serde_json::Value>,
}

impl Default for NodeMeta {
    fn default() -> Self {
        Self {
            collapsed: false,
            deleted: false,
            attrs: HashMap::new(),
        }
    }
}

/// Complete node structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Unique identifier
    pub id: NodeId,
    /// Parent node ID
    pub parent: Option<NodeId>,
    /// Child node IDs
    #[serde(default)]
    pub children: Vec<NodeId>,
    /// Node type
    pub node_type: NodeType,
    /// Node content
    pub content: NodeContent,
    /// Node metadata
    #[serde(default)]
    pub meta: NodeMeta,
}

impl Node {
    /// Create a new node
    pub fn new(id: NodeId, node_type: NodeType, content: NodeContent) -> Self {
        Self {
            id,
            parent: None,
            children: Vec::new(),
            node_type,
            content,
            meta: NodeMeta::default(),
        }
    }

    /// Check if node is a leaf (no children)
    pub fn is_leaf(&self) -> bool {
        self.node_type.is_leaf() || self.children.is_empty()
    }

    /// Check if node can contain children
    pub fn can_have_children(&self) -> bool {
        matches!(
            self.content,
            NodeContent::Mixed(_) | NodeContent::Empty
        )
    }
}
