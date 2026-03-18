//! Node definitions and implementations
//!
//! Defines the core Node enum for block-level and text nodes,
//! with serialization, deserialization, and content size calculation.

use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::fmt;

/// Represents the type of a node
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    /// Block-level node types (paragraph, heading, blockquote, etc.)
    Block(String),
    /// Text node with inline formatting
    Text,
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeType::Block(name) => write!(f, "{}", name),
            NodeType::Text => write!(f, "text"),
        }
    }
}

/// Content of a node - either child nodes or text
#[derive(Debug, Clone, PartialEq)]
pub enum NodeContent {
    /// Block nodes contain child nodes
    Nodes(Vec<Node>),
    /// Text nodes contain a string and optional marks
    Text(Arc<TextNodeContent>),
}

impl serde::Serialize for NodeContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        match self {
            NodeContent::Nodes(nodes) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("nodes", nodes)?;
                map.end()
            }
            NodeContent::Text(tc) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("text", tc.as_ref())?;
                map.end()
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for NodeContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ContentVariant {
            #[serde(default)]
            nodes: Option<Vec<Node>>,
            #[serde(default)]
            text: Option<TextNodeContent>,
        }
        let variant = ContentVariant::deserialize(deserializer)?;
        match (variant.nodes, variant.text) {
            (Some(nodes), None) => Ok(NodeContent::Nodes(nodes)),
            (None, Some(tc)) => Ok(NodeContent::Text(Arc::new(tc))),
            _ => Err(serde::de::Error::custom("expected either `nodes` or `text` content")),
        }
    }
}

/// Text node specific content
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextNodeContent {
    /// The actual text content
    pub text: String,
    /// Marks (formatting) applied to this text
    pub marks: Vec<Mark>,
}

/// A mark represents inline formatting (bold, italic, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Mark {
    /// Type of the mark (e.g., "bold", "italic", "link")
    pub type_name: String,
    /// Optional attributes for the mark
    pub attrs: Option<serde_json::Value>,
}

/// Calculated content size for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentSize {
    /// Total size including all content
    pub size: usize,
    /// Size of the content itself (excluding wrapper)
    pub content_size: usize,
}

impl ContentSize {
    /// Create a new content size
    pub fn new(size: usize, content_size: usize) -> Self {
        Self { size, content_size }
    }

    /// Zero size
    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

/// The core Node struct
///
/// Represents both block-level nodes (paragraphs, headings, etc.)
/// and text nodes with inline formatting.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    /// Type of this node
    pub type_: NodeType,
    /// Optional type for block nodes
    pub block_type: Option<String>,
    /// Node content
    pub content: NodeContent,
    /// Node attributes (id, styles, etc.)
    pub attrs: Option<serde_json::Value>,
    /// Whether this is a leaf node (no children)
    pub is_leaf: bool,
    /// Marks for inline nodes
    pub marks: Vec<Mark>,
}

impl serde::Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Node", 6)?;
        s.serialize_field("type", &self.type_)?;
        s.serialize_field("block_type", &self.block_type)?;
        s.serialize_field("content", &self.content)?;
        s.serialize_field("attrs", &self.attrs)?;
        s.serialize_field("is_leaf", &self.is_leaf)?;
        s.serialize_field("marks", &self.marks)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Type,
            BlockType,
            Content,
            Attrs,
            IsLeaf,
            Marks,
        }

        struct NodeVisitor;

        impl<'de> serde::de::Visitor<'de> for NodeVisitor {
            type Value = Node;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Node struct")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Node, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut type_ = None;
                let mut block_type = None;
                let mut content = None;
                let mut attrs = None;
                let mut is_leaf = None;
                let mut marks = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Type => type_ = Some(map.next_value()?),
                        Field::BlockType => block_type = Some(map.next_value()?),
                        Field::Content => content = Some(map.next_value()?),
                        Field::Attrs => attrs = Some(map.next_value()?),
                        Field::IsLeaf => is_leaf = Some(map.next_value()?),
                        Field::Marks => marks = Some(map.next_value()?),
                    }
                }

                Ok(Node {
                    type_: type_.unwrap(),
                    block_type,
                    content: content.unwrap(),
                    attrs,
                    is_leaf: is_leaf.unwrap_or(false),
                    marks: marks.unwrap_or_default(),
                })
            }
        }

        const FIELDS: &[&str] = &["type", "block_type", "content", "attrs", "is_leaf", "marks"];
        deserializer.deserialize_struct("Node", FIELDS, NodeVisitor)
    }
}

impl Node {
    pub fn char_to_byte_index(text: &str, char_index: usize) -> usize {
        text.char_indices()
            .nth(char_index)
            .map(|(byte_index, _)| byte_index)
            .unwrap_or(text.len())
    }

    pub fn char_len(text: &str) -> usize {
        text.chars().count()
    }

    /// Create a new block node
    pub fn new_block(block_type: impl Into<String>, content: Vec<Node>) -> Self {
        let block_type_str = block_type.into();
        Self {
            type_: NodeType::Block(block_type_str.clone()),
            block_type: Some(block_type_str),
            content: NodeContent::Nodes(content),
            attrs: None,
            is_leaf: false,
            marks: Vec::new(),
        }
    }

    /// Create a new text node
    pub fn new_text(text: impl Into<String>) -> Self {
        Self {
            type_: NodeType::Text,
            block_type: None,
            content: NodeContent::Text(Arc::new(TextNodeContent {
                text: text.into(),
                marks: Vec::new(),
            })),
            attrs: None,
            is_leaf: true,
            marks: Vec::new(),
        }
    }

    /// Create a new text node with marks
    pub fn new_text_with_marks(text: impl Into<String>, marks: Vec<Mark>) -> Self {
        Self {
            type_: NodeType::Text,
            block_type: None,
            content: NodeContent::Text(Arc::new(TextNodeContent {
                text: text.into(),
                marks,
            })),
            attrs: None,
            is_leaf: true,
            marks: Vec::new(),
        }
    }

    /// Get the node type name
    pub fn type_name(&self) -> &str {
        match &self.type_ {
            NodeType::Block(name) => name.as_str(),
            NodeType::Text => "text",
        }
    }

    /// Check if this is a block node
    pub fn is_block(&self) -> bool {
        matches!(self.type_, NodeType::Block(_))
    }

    /// Check if this is a text node
    pub fn is_text(&self) -> bool {
        matches!(self.type_, NodeType::Text)
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.is_leaf
    }

    /// Get child nodes if this is a block node
    pub fn children(&self) -> Option<&[Node]> {
        match &self.content {
            NodeContent::Nodes(nodes) => Some(nodes.as_slice()),
            NodeContent::Text(_) => None,
        }
    }

    /// Get mutable child nodes if this is a block node
    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        match &mut self.content {
            NodeContent::Nodes(nodes) => Some(nodes),
            NodeContent::Text(_) => None,
        }
    }

    /// Get text content if this is a text node
    pub fn text(&self) -> Option<&str> {
        match &self.content {
            NodeContent::Nodes(_) => None,
            NodeContent::Text(tc) => Some(tc.text.as_str()),
        }
    }

    /// Collect all descendant text into a single plain-text string.
    pub fn text_content(&self) -> String {
        match &self.content {
            NodeContent::Nodes(nodes) => nodes.iter().map(Node::text_content).collect(),
            NodeContent::Text(tc) => tc.text.clone(),
        }
    }

    /// Collect descendant text using the canonical editor plain-text view.
    ///
    /// For `doc` nodes in the minimal schema, top-level paragraphs are joined by `\n`.
    pub fn plain_text(&self) -> String {
        if self.type_name() == "doc" {
            return self.paragraph_texts().join("\n");
        }

        self.text_content()
    }

    /// Return the length of the canonical plain-text view.
    pub fn plain_text_len(&self) -> usize {
        Self::char_len(&self.plain_text())
    }

    /// Return a copy of this node tree with its plain-text content replaced.
    ///
    /// This supports the current minimal editor schema subset:
    /// `doc -> paragraph -> text*` and standalone text nodes.
    pub fn with_text_content(&self, text: impl Into<String>) -> Self {
        let text = text.into();
        match &self.content {
            NodeContent::Text(tc) => Self {
                type_: self.type_.clone(),
                block_type: self.block_type.clone(),
                content: NodeContent::Text(Arc::new(TextNodeContent {
                    text,
                    marks: tc.marks.clone(),
                })),
                attrs: self.attrs.clone(),
                is_leaf: true,
                marks: self.marks.clone(),
            },
            NodeContent::Nodes(_) if self.type_name() == "doc" => {
                let block_type = self
                    .children()
                    .and_then(|children| children.first())
                    .and_then(|child| child.block_type.clone())
                    .unwrap_or_else(|| "paragraph".to_string());
                Self::new_block("doc", vec![Node::new_block(block_type, vec![Node::new_text(text)])])
            }
            NodeContent::Nodes(_) => Self::new_block(self.type_name(), vec![Node::new_text(text)]),
        }
    }

    /// Return a copy of this node tree with its canonical plain-text content replaced.
    ///
    /// For `doc` nodes, `\n` is interpreted as a paragraph separator.
    pub fn with_plain_text(&self, text: impl Into<String>) -> Self {
        let text = text.into();
        if self.type_name() == "doc" {
            let paragraphs = text.split('\n').map(str::to_string).collect();
            return Self::from_paragraph_texts(paragraphs);
        }

        self.with_text_content(text)
    }

    /// Return the document's paragraph texts for the minimal supported schema.
    pub fn paragraph_texts(&self) -> Vec<String> {
        if self.type_name() == "doc" {
            return self
                .children()
                .map(|children| children.iter().map(Node::text_content).collect())
                .unwrap_or_default();
        }

        vec![self.text_content()]
    }

    /// Locate a flat text position within the document's paragraph list.
    pub fn paragraph_at(&self, pos: usize) -> Option<(usize, usize)> {
        let paragraphs = self.paragraph_texts();
        let mut start = 0;

        for (index, paragraph) in paragraphs.iter().enumerate() {
            let end = start + Self::char_len(paragraph);
            if pos <= end {
                return Some((index, pos.saturating_sub(start)));
            }
            start = end + 1;
        }

        paragraphs
            .len()
            .checked_sub(1)
            .map(|last| (last, Self::char_len(&paragraphs[last])))
    }

    /// Rebuild a document from paragraph texts using the minimal schema.
    pub fn from_paragraph_texts(paragraphs: Vec<String>) -> Self {
        let paragraphs = if paragraphs.is_empty() {
            vec![String::new()]
        } else {
            paragraphs
        };

        Self::new_block(
            "doc",
            paragraphs
                .into_iter()
                .map(|text| Node::new_block("paragraph", vec![Node::new_text(text)]))
                .collect(),
        )
    }

    /// Apply mark changes over a flat text range in the minimal schema.
    pub fn with_mark_range(&self, from: usize, to: usize, add: &[Mark], remove: &[String]) -> Self {
        fn apply_to_node(node: &Node, from: usize, to: usize, offset: &mut usize, add: &[Mark], remove: &[String]) -> Node {
            match &node.content {
                NodeContent::Text(tc) => {
                    let start = *offset;
                    let end = start + Node::char_len(&tc.text);
                    *offset = end;

                    if to <= start || from >= end {
                        return node.clone();
                    }

                    let mut marks = tc.marks.clone();
                    marks.retain(|mark| !remove.iter().any(|mark_type| mark.type_name == *mark_type));
                    for mark in add {
                        if !marks.iter().any(|existing| existing.type_name == mark.type_name) {
                            marks.push(mark.clone());
                        }
                    }

                    Node::new_text_with_marks(tc.text.clone(), marks)
                }
                NodeContent::Nodes(children) => {
                    let mut next_children = Vec::with_capacity(children.len());
                    for (index, child) in children.iter().enumerate() {
                        next_children.push(apply_to_node(child, from, to, offset, add, remove));
                        if node.type_name() == "doc" && index + 1 < children.len() {
                            *offset += 1;
                        }
                    }
                    Node::new_block(node.type_name(), next_children)
                }
            }
        }

        let mut offset = 0;
        apply_to_node(self, from, to, &mut offset, add, remove)
    }

    /// Calculate the content size of this node
    pub fn content_size(&self) -> ContentSize {
        match &self.content {
            NodeContent::Nodes(nodes) => {
                let mut total_size = 0;
                let mut total_content = 0;
                for child in nodes {
                    let child_size = child.content_size();
                    total_size += child_size.size;
                    total_content += child_size.content_size;
                }
                // Add size for the wrapper markers (e.g., block tags)
                total_size += 2; // Basic wrapper overhead
                ContentSize::new(total_size, total_content)
            }
            NodeContent::Text(tc) => {
                let size = Self::char_len(&tc.text);
                ContentSize::new(size, size)
            }
        }
    }

    /// Get the total size of this node
    pub fn size(&self) -> usize {
        self.content_size().size
    }

    /// Get the content size only
    pub fn content_length(&self) -> usize {
        self.content_size().content_size
    }

    /// Count the number of child nodes
    pub fn child_count(&self) -> usize {
        match &self.content {
            NodeContent::Nodes(nodes) => nodes.len(),
            NodeContent::Text(_) => 0,
        }
    }

    /// Get a child at a specific index
    pub fn child(&self, index: usize) -> Option<&Node> {
        match &self.content {
            NodeContent::Nodes(nodes) => nodes.get(index),
            NodeContent::Text(_) => None,
        }
    }

    /// Find the position where a child starts
    pub fn child_start_pos(&self, child_index: usize) -> usize {
        match &self.content {
            NodeContent::Nodes(nodes) => {
                let mut pos = 0;
                for (i, child) in nodes.iter().enumerate() {
                    if i == child_index {
                        break;
                    }
                    pos += child.content_size().size;
                }
                pos
            }
            NodeContent::Text(_) => 0,
        }
    }

    /// Serialize this node to JSON
    pub fn to_json(&self) -> serde_json::Value {
        let content = match &self.content {
            NodeContent::Nodes(nodes) => serde_json::json!({
                "nodes": nodes.iter().map(Node::to_json).collect::<Vec<_>>()
            }),
            NodeContent::Text(tc) => serde_json::json!({
                "text": {
                    "text": tc.text,
                    "marks": tc.marks,
                }
            }),
        };

        serde_json::json!({
            "type": self.type_,
            "block_type": self.block_type,
            "content": content,
            "attrs": self.attrs,
            "is_leaf": self.is_leaf,
            "marks": self.marks,
        })
    }

    /// Deserialize a node from JSON
    pub fn from_json(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        let obj = value.as_object().ok_or_else(|| {
            serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, "node must be an object"))
        })?;

        let type_ = serde_json::from_value(obj.get("type").cloned().unwrap_or(serde_json::Value::Null))?;
        let block_type = serde_json::from_value(
            obj.get("block_type").cloned().unwrap_or(serde_json::Value::Null),
        )?;
        let attrs = serde_json::from_value(obj.get("attrs").cloned().unwrap_or(serde_json::Value::Null))?;
        let is_leaf = serde_json::from_value(
            obj.get("is_leaf").cloned().unwrap_or(serde_json::Value::Bool(false)),
        )?;
        let marks = serde_json::from_value(
            obj.get("marks").cloned().unwrap_or_else(|| serde_json::json!([])),
        )?;

        let content_value = obj.get("content").cloned().unwrap_or(serde_json::Value::Null);
        let content_obj = content_value.as_object().ok_or_else(|| {
            serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, "content must be an object"))
        })?;

        let content = if let Some(nodes_value) = content_obj.get("nodes") {
            let nodes_json: Vec<serde_json::Value> = serde_json::from_value(nodes_value.clone())?;
            let mut nodes = Vec::with_capacity(nodes_json.len());
            for node_json in nodes_json {
                nodes.push(Node::from_json(node_json)?);
            }
            NodeContent::Nodes(nodes)
        } else if let Some(text_value) = content_obj.get("text") {
            let text_content: TextNodeContent = serde_json::from_value(text_value.clone())?;
            NodeContent::Text(Arc::new(text_content))
        } else {
            return Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "content must contain either `nodes` or `text`",
            )));
        };

        Ok(Self {
            type_,
            block_type,
            content,
            attrs,
            is_leaf,
            marks,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_new_block() {
        let node = Node::new_block("paragraph", vec![
            Node::new_text("Hello"),
            Node::new_text("World"),
        ]);

        assert!(node.is_block());
        assert!(!node.is_text());
        assert!(!node.is_leaf());
        assert_eq!(node.type_name(), "paragraph");
        assert_eq!(node.child_count(), 2);
    }

    #[test]
    fn test_node_new_text() {
        let node = Node::new_text("Hello World");

        assert!(!node.is_block());
        assert!(node.is_text());
        assert!(node.is_leaf());
        assert_eq!(node.text(), Some("Hello World"));
        assert_eq!(node.content_length(), 11);
    }

    #[test]
    fn test_text_content_round_trip() {
        let doc = Node::new_block(
            "doc",
            vec![Node::new_block(
                "paragraph",
                vec![Node::new_text("Hello"), Node::new_text(" World")],
            )],
        );

        assert_eq!(doc.text_content(), "Hello World");

        let updated = doc.with_text_content("Hi Rust");
        assert_eq!(updated.text_content(), "Hi Rust");
        assert_eq!(updated.type_name(), "doc");
    }

    #[test]
    fn test_plain_text_round_trip() {
        let doc = Node::from_paragraph_texts(vec!["Hello".to_string(), "World".to_string()]);

        assert_eq!(doc.plain_text(), "Hello\nWorld");
        assert_eq!(doc.plain_text_len(), 11);

        let updated = doc.with_plain_text("Hi\nRust");
        assert_eq!(updated.paragraph_texts(), vec!["Hi".to_string(), "Rust".to_string()]);
    }

    #[test]
    fn test_plain_text_len_counts_characters() {
        let doc = Node::from_paragraph_texts(vec!["你a".to_string(), "界".to_string()]);

        assert_eq!(doc.plain_text(), "你a\n界");
        assert_eq!(doc.plain_text_len(), 4);
        assert_eq!(Node::char_to_byte_index(&doc.plain_text(), 1), "你".len());
    }

    #[test]
    fn test_paragraph_helpers() {
        let doc = Node::from_paragraph_texts(vec!["Hello".to_string(), "World".to_string()]);

        assert_eq!(doc.paragraph_texts(), vec!["Hello".to_string(), "World".to_string()]);
        assert_eq!(doc.paragraph_at(0), Some((0, 0)));
        assert_eq!(doc.paragraph_at(5), Some((0, 5)));
        assert_eq!(doc.paragraph_at(6), Some((1, 0)));
        assert_eq!(doc.paragraph_at(7), Some((1, 1)));
    }

    #[test]
    fn test_paragraph_at_uses_character_offsets() {
        let doc = Node::from_paragraph_texts(vec!["你a".to_string(), "界".to_string()]);

        assert_eq!(doc.paragraph_at(0), Some((0, 0)));
        assert_eq!(doc.paragraph_at(1), Some((0, 1)));
        assert_eq!(doc.paragraph_at(2), Some((0, 2)));
        assert_eq!(doc.paragraph_at(3), Some((1, 0)));
        assert_eq!(doc.paragraph_at(4), Some((1, 1)));
    }

    #[test]
    fn test_with_mark_range() {
        let doc = Node::new_block(
            "doc",
            vec![Node::new_block(
                "paragraph",
                vec![Node::new_text("Hello"), Node::new_text("World")],
            )],
        );
        let marked = doc.with_mark_range(
            0,
            10,
            &[Mark { type_name: "bold".to_string(), attrs: None }],
            &[],
        );

        let children = marked.children().unwrap()[0].children().unwrap();
        for child in children {
            match &child.content {
                NodeContent::Text(tc) => assert_eq!(tc.marks[0].type_name, "bold"),
                NodeContent::Nodes(_) => panic!("expected text child"),
            }
        }
    }

    #[test]
    fn test_node_content_size() {
        let text1 = Node::new_text("Hello");
        let text2 = Node::new_text(" ");
        let text3 = Node::new_text("World");

        let paragraph = Node::new_block("paragraph", vec![text1, text2, text3]);

        // Content size should be sum of text lengths
        assert_eq!(paragraph.content_length(), 11);
    }

    #[test]
    fn test_node_serialization() {
        let node = Node::new_block("heading", vec![Node::new_text("Title")]);

        let json = node.to_json();
        let deserialized = Node::from_json(json).expect("Failed to deserialize");

        assert_eq!(node.type_name(), deserialized.type_name());
        assert_eq!(node.child_count(), deserialized.child_count());
    }

    #[test]
    fn test_node_with_marks() {
        let marks = vec![
            Mark { type_name: "bold".to_string(), attrs: None },
        ];
        let node = Node::new_text_with_marks("bold text", marks.clone());

        if let NodeContent::Text(tc) = &node.content {
            assert_eq!(tc.marks, marks);
        }
    }
}
