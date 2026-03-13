//! Selection handling
//!
//! Defines selection types: text selection and node selection.

use model::{Node, ResolvedPos};
use serde::{Serialize, Deserialize};
use std::fmt;

/// A selection range with anchor and head
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionRange {
    /// The anchor point (where selection started)
    pub anchor: usize,
    /// The head point (where selection ends)
    pub head: usize,
    /// The resolved anchor position
    pub resolved_anchor: Option<ResolvedPos>,
    /// The resolved head position
    pub resolved_head: Option<ResolvedPos>,
}

impl SelectionRange {
    /// Create a new range
    pub fn new(anchor: usize, head: usize) -> Self {
        Self {
            anchor,
            head,
            resolved_anchor: None,
            resolved_head: None,
        }
    }

    /// Create a collapsed range (cursor)
    pub fn collapse(anchor: usize) -> Self {
        Self::new(anchor, anchor)
    }

    /// Get the minimum position
    pub fn from(&self) -> usize {
        self.anchor.min(self.head)
    }

    /// Get the maximum position
    pub fn to(&self) -> usize {
        self.anchor.max(self.head)
    }

    /// Check if collapsed (cursor position)
    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.head
    }

    /// Check if this is a backwards selection
    pub fn is_backwards(&self) -> bool {
        self.head < self.anchor
    }

    /// Get the resolved positions
    pub fn resolve(&self, doc: &Node) -> Self {
        Self {
            anchor: self.anchor,
            head: self.head,
            resolved_anchor: Some(ResolvedPos::from_pos(doc, self.anchor)),
            resolved_head: Some(ResolvedPos::from_pos(doc, self.head)),
        }
    }

    /// Map this range through a step map
    pub fn map(&self, from: usize, to: usize) -> Self {
        Self {
            anchor: to,
            head: if self.is_collapsed() { to } else { from },
            resolved_anchor: None,
            resolved_head: None,
        }
    }
}

impl fmt::Display for SelectionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_collapsed() {
            write!(f, "Cursor({})", self.anchor)
        } else {
            write!(f, "Range({} → {})", self.anchor, self.head)
        }
    }
}

/// Text selection (character/word selection)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextSelection {
    /// The main selection range
    pub ranges: Vec<SelectionRange>,
}

impl TextSelection {
    /// Create a text selection with one range
    pub fn at(doc: &Node, pos: usize) -> Self {
        let range = SelectionRange::collapse(pos).resolve(doc);
        Self { ranges: vec![range] }
    }

    /// Create a text selection with a range
    pub fn create(doc: &Node, anchor: usize, head: usize) -> Self {
        let range = SelectionRange::new(anchor, head).resolve(doc);
        Self { ranges: vec![range] }
    }

    /// Get the anchor position
    pub fn anchor(&self) -> usize {
        self.ranges.first().map(|r| r.anchor).unwrap_or(0)
    }

    /// Get the head position
    pub fn head(&self) -> usize {
        self.ranges.first().map(|r| r.head).unwrap_or(0)
    }

    /// Get the from position
    pub fn from(&self) -> usize {
        self.ranges.first().map(|r| r.from()).unwrap_or(0)
    }

    /// Get the to position
    pub fn to(&self) -> usize {
        self.ranges.first().map(|r| r.to()).unwrap_or(0)
    }

    /// Check if collapsed
    pub fn is_collapsed(&self) -> bool {
        self.ranges.iter().all(|r| r.is_collapsed())
    }

    /// Get all ranges
    pub fn ranges(&self) -> &[SelectionRange] {
        &self.ranges
    }

    /// Get the primary range
    pub fn primary(&self) -> Option<&SelectionRange> {
        self.ranges.first()
    }

    /// Resolve positions for a document
    pub fn resolve(&self, doc: &Node) -> Self {
        Self {
            ranges: self.ranges.iter().map(|r| r.resolve(doc)).collect(),
        }
    }

    /// Map selection through changes
    pub fn map(&self, from: usize, to: usize) -> Self {
        Self {
            ranges: self.ranges.iter().map(|r| r.map(from, to)).collect(),
        }
    }

    /// Get the resolved anchor position
    pub fn resolved_anchor(&self) -> Option<&ResolvedPos> {
        self.ranges.first().and_then(|r| r.resolved_anchor.as_ref())
    }

    /// Get the resolved head position
    pub fn resolved_head(&self) -> Option<&ResolvedPos> {
        self.ranges.first().and_then(|r| r.resolved_head.as_ref())
    }
}

impl Default for TextSelection {
    fn default() -> Self {
        Self { ranges: vec![SelectionRange::collapse(0)] }
    }
}

impl fmt::Display for TextSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_collapsed() {
            write!(f, "TextSelection({})", self.head())
        } else {
            write!(f, "TextSelection({} → {})", self.from(), self.to())
        }
    }
}

/// Node selection (selecting entire nodes)
#[derive(Debug, Clone, PartialEq)]
pub struct NodeSelection {
    /// The selected node's position
    pub pos: usize,
    /// The depth of the node
    pub depth: usize,
}

impl NodeSelection {
    /// Create a node selection
    pub fn new(pos: usize, depth: usize) -> Self {
        Self { pos, depth }
    }

    /// Create a node selection from a node
    pub fn from_node(_doc: &Node, node: &Node, index: usize) -> Self {
        let pos = node.child_start_pos(index);
        Self { pos, depth: 1 }
    }

    /// Get the position
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Get the depth
    pub fn depth(&self) -> usize {
        self.depth
    }
}

impl Default for NodeSelection {
    fn default() -> Self {
        Self { pos: 0, depth: 0 }
    }
}

impl fmt::Display for NodeSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeSelection(pos={}, depth={})", self.pos, self.depth)
    }
}

impl serde::Serialize for NodeSelection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("NodeSelection", 2)?;
        s.serialize_field("pos", &self.pos)?;
        s.serialize_field("depth", &self.depth)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for NodeSelection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Pos,
            Depth,
        }

        struct NodeSelectionVisitor;

        impl<'de> serde::de::Visitor<'de> for NodeSelectionVisitor {
            type Value = NodeSelection;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a NodeSelection struct")
            }

            fn visit_map<V>(self, mut map: V) -> Result<NodeSelection, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut pos = None;
                let mut depth = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Pos => pos = Some(map.next_value()?),
                        Field::Depth => depth = Some(map.next_value()?),
                    }
                }

                Ok(NodeSelection {
                    pos: pos.unwrap_or(0),
                    depth: depth.unwrap_or(0),
                })
            }
        }

        const FIELDS: &[&str] = &["pos", "depth"];
        deserializer.deserialize_struct("NodeSelection", FIELDS, NodeSelectionVisitor)
    }
}

/// Trait for all selections
#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    Text(TextSelection),
    Node(NodeSelection),
}

impl Selection {
    /// Create a text selection
    pub fn text(text: TextSelection) -> Self {
        Self::Text(text)
    }

    /// Create a node selection
    pub fn node(node: NodeSelection) -> Self {
        Self::Node(node)
    }

    /// Check if this is a text selection
    pub fn is_text(&self) -> bool {
        matches!(self, Selection::Text(_))
    }

    /// Check if this is a node selection
    pub fn is_node(&self) -> bool {
        matches!(self, Selection::Node(_))
    }

    /// Get as text selection
    pub fn as_text(&self) -> Option<&TextSelection> {
        match self {
            Selection::Text(t) => Some(t),
            Selection::Node(_) => None,
        }
    }

    /// Get as node selection
    pub fn as_node(&self) -> Option<&NodeSelection> {
        match self {
            Selection::Text(_) => None,
            Selection::Node(n) => Some(n),
        }
    }

    /// Get the anchor position
    pub fn anchor(&self) -> usize {
        match self {
            Selection::Text(t) => t.anchor(),
            Selection::Node(n) => n.pos(),
        }
    }

    /// Get the head position
    pub fn head(&self) -> usize {
        match self {
            Selection::Text(t) => t.head(),
            Selection::Node(n) => n.pos(),
        }
    }

    /// Check if collapsed
    pub fn is_collapsed(&self) -> bool {
        match self {
            Selection::Text(t) => t.is_collapsed(),
            Selection::Node(_) => false,
        }
    }

    /// Get the from position
    pub fn from(&self) -> usize {
        match self {
            Selection::Text(t) => t.from(),
            Selection::Node(n) => n.pos(),
        }
    }

    /// Get the to position
    pub fn to(&self) -> usize {
        match self {
            Selection::Text(t) => t.to(),
            Selection::Node(n) => n.pos() + 1,
        }
    }

    /// Resolve positions for a document
    pub fn resolve(&self, doc: &Node) -> Self {
        match self {
            Selection::Text(t) => Selection::Text(t.resolve(doc)),
            Selection::Node(n) => Selection::Node(n.clone()),
        }
    }

    /// Map selection through changes
    pub fn map(&self, from: usize, to: usize) -> Self {
        match self {
            Selection::Text(t) => Selection::Text(t.map(from, to)),
            Selection::Node(n) => Selection::Node(NodeSelection::new(to, n.depth)),
        }
    }
}

impl fmt::Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Selection::Text(t) => write!(f, "{}", t),
            Selection::Node(n) => write!(f, "{}", n),
        }
    }
}

impl serde::Serialize for Selection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Selection::Text(t) => {
                use serde::ser::SerializeStruct;
                let mut s = serializer.serialize_struct("Selection", 2)?;
                s.serialize_field("type", "text")?;
                s.serialize_field("ranges", &t.ranges)?;
                s.end()
            }
            Selection::Node(n) => {
                use serde::ser::SerializeStruct;
                let mut s = serializer.serialize_struct("Selection", 2)?;
                s.serialize_field("type", "node")?;
                s.serialize_field("pos", &n.pos)?;
                s.serialize_field("depth", &n.depth)?;
                s.end()
            }
        }
    }
}

impl<'de> serde::Deserialize<'de> for Selection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Type,
            Ranges,
            Pos,
            Depth,
        }

        struct SelectionVisitor;

        impl<'de> serde::de::Visitor<'de> for SelectionVisitor {
            type Value = Selection;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Selection struct")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Selection, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut type_ = None;
                let mut ranges = None;
                let mut pos = None;
                let mut depth = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Type => {
                            type_ = Some(map.next_value::<String>()?);
                        }
                        Field::Ranges => {
                            ranges = Some(map.next_value::<Vec<SelectionRange>>()?);
                        }
                        Field::Pos => {
                            pos = Some(map.next_value::<usize>()?);
                        }
                        Field::Depth => {
                            depth = Some(map.next_value::<usize>()?);
                        }
                    }
                }

                match type_.as_deref() {
                    Some("text") => {
                        Ok(Selection::Text(TextSelection {
                            ranges: ranges.unwrap_or_default(),
                        }))
                    }
                    Some("node") => {
                        Ok(Selection::Node(NodeSelection {
                            pos: pos.unwrap_or(0),
                            depth: depth.unwrap_or(0),
                        }))
                    }
                    _ => Err(serde::de::Error::custom("Invalid selection type")),
                }
            }
        }

        const FIELDS: &[&str] = &["type", "ranges", "pos", "depth"];
        deserializer.deserialize_struct("Selection", FIELDS, SelectionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_range_collapsed() {
        let range = SelectionRange::collapse(5);
        assert!(range.is_collapsed());
        assert_eq!(range.anchor, 5);
        assert_eq!(range.head, 5);
    }

    #[test]
    fn test_selection_range_not_collapsed() {
        let range = SelectionRange::new(5, 10);
        assert!(!range.is_collapsed());
        assert_eq!(range.from(), 5);
        assert_eq!(range.to(), 10);
    }

    #[test]
    fn test_selection_range_backwards() {
        let range = SelectionRange::new(10, 5);
        assert!(range.is_backwards());
    }

    #[test]
    fn test_text_selection() {
        let doc = Node::new_block("paragraph", vec![Node::new_text("Hello World")]);
        let sel = TextSelection::create(&doc, 0, 5);

        assert_eq!(sel.anchor(), 0);
        assert_eq!(sel.head(), 5);
        assert!(!sel.is_collapsed());
    }

    #[test]
    fn test_text_selection_resolve() {
        let doc = Node::new_block("paragraph", vec![Node::new_text("Hello")]);
        let sel = TextSelection::create(&doc, 0, 5).resolve(&doc);

        assert!(sel.resolved_anchor().is_some());
        assert!(sel.resolved_head().is_some());
    }

    #[test]
    fn test_node_selection() {
        let sel = NodeSelection::new(10, 2);
        assert_eq!(sel.pos(), 10);
        assert_eq!(sel.depth(), 2);
    }

    #[test]
    fn test_selection_text() {
        let doc = Node::new_block("paragraph", vec![Node::new_text("Hello")]);
        let sel = Selection::text(TextSelection::at(&doc, 5));

        assert!(sel.is_text());
        assert!(!sel.is_node());
        assert_eq!(sel.head(), 5);
    }

    #[test]
    fn test_selection_node() {
        let sel = Selection::node(NodeSelection::new(10, 1));

        assert!(!sel.is_text());
        assert!(sel.is_node());
    }

    #[test]
    fn test_selection_mapping() {
        let sel = Selection::text(TextSelection::create(&Node::new_text("doc"), 0, 5));
        let mapped = sel.map(5, 10);

        assert_eq!(mapped.from(), 5);
        assert_eq!(mapped.to(), 10);
    }
}
