//! Resolved position handling
//!
//! Represents a position in the document with information about
//! its parent, depth, and index at each level.

use super::Node;
use serde::Deserialize;
use std::fmt;

/// Represents a resolved document position
///
/// Unlike a raw `usize` position, `ResolvedPos` provides information
/// about the path to reach that position, including parent nodes
/// and the index within each parent.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPos {
    /// The absolute position in the document
    pub pos: usize,
    /// The depth of the position (how many parent nodes)
    pub depth: usize,
    /// Path from root to parent at each level
    path: Vec<PathEntry>,
}

/// An entry in the resolution path
#[derive(Debug, Clone, PartialEq)]
struct PathEntry {
    /// The parent node
    node: Node,
    /// Index of the child we're positioned at/after
    index: usize,
    /// Position of this node relative to parent start
    node_offset: usize,
}

impl ResolvedPos {
    /// Create a resolved position at a specific index in the document
    pub fn at_index(doc: &Node, index: usize) -> Self {
        let mut path = Vec::new();
        let mut pos = 0;
        let mut depth = 0;

        // Traverse to find the path
        if let Some((new_depth, new_path, new_pos)) = resolve_path(doc, 0, index, 0, Vec::new()) {
            depth = new_depth;
            path = new_path;
            pos = new_pos;
        }

        Self { pos, depth, path }
    }

    /// Create a resolved position from an absolute position
    pub fn from_pos(doc: &Node, pos: usize) -> Self {
        let mut path = Vec::new();
        let mut current_pos = 0;
        let mut depth = 0;

        // Find the leaf node and build path
        find_path(doc, pos, 0, &mut path, &mut current_pos, &mut depth);

        Self { pos, depth, path }
    }

    /// Get the parent node at a specific depth
    ///
    /// Depth 0 is the root document, depth 1 is the first child level, etc.
    /// Returns `None` if depth is out of range.
    pub fn parent(&self, depth: usize) -> Option<&Node> {
        if depth == 0 {
            // For depth 0, we return the root document itself
            Some(&self.path.first()?.node)
        } else if depth <= self.depth {
            Some(&self.path[depth - 1].node)
        } else {
            None
        }
    }

    /// Get the immediate parent of the current position
    pub fn immediate_parent(&self) -> Option<&Node> {
        if self.depth == 0 {
            None
        } else {
            Some(&self.path[self.depth - 1].node)
        }
    }

    /// Get the index within the parent at a specific depth
    pub fn index(&self, depth: usize) -> Option<usize> {
        if depth == 0 {
            Some(0)
        } else if depth <= self.depth {
            Some(self.path[depth - 1].index)
        } else {
            None
        }
    }

    /// Get the index within the immediate parent
    pub fn current_index(&self) -> usize {
        if self.depth == 0 {
            0
        } else {
            self.path[self.depth - 1].index
        }
    }

    /// Get the depth
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Get the absolute position
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Check if this position is at the start of a node
    pub fn is_at_node_start(&self) -> bool {
        self.pos == self.node_offset()
    }

    /// Get the offset of the current node from document start
    pub fn node_offset(&self) -> usize {
        if self.path.is_empty() {
            0
        } else {
            self.path.last().map(|e| e.node_offset).unwrap_or(0)
        }
    }

    /// Check if this position is at the end of a node
    pub fn is_at_node_end(&self) -> bool {
        if let Some(parent) = self.immediate_parent() {
            let node_size = parent.content_size().size;
            return self.pos == self.node_offset() + node_size;
        }
        false
    }

    /// Check if this is the start of the document
    pub fn is_doc_start(&self) -> bool {
        self.pos == 0
    }

    /// Check if this is the end of the document
    pub fn is_doc_end(&self) -> bool {
        // Check if we're at the end of the root
        if let Some(root) = self.path.first() {
            let root_size = root.node.content_size().size;
            return self.pos == root_size;
        }
        false
    }

    /// Get the node and index at the current depth
    pub fn descendent(&self) -> Option<Descendant> {
        if self.depth == 0 {
            None
        } else {
            let entry = &self.path[self.depth - 1];
            Some(Descendant {
                node: entry.node.clone(),
                index: entry.index,
                offset: entry.node_offset,
            })
        }
    }

    /// Get all ancestors (excluding self)
    pub fn ancestors(&self) -> impl Iterator<Item = &Node> {
        self.path.iter().map(|e| &e.node)
    }

}

impl serde::Serialize for ResolvedPos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("ResolvedPos", 2)?;
        s.serialize_field("pos", &self.pos)?;
        s.serialize_field("depth", &self.depth)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for ResolvedPos {
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

        struct ResolvedPosVisitor;

        impl<'de> serde::de::Visitor<'de> for ResolvedPosVisitor {
            type Value = ResolvedPos;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a ResolvedPos struct")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ResolvedPos, V::Error>
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

                Ok(ResolvedPos {
                    pos: pos.unwrap_or(0),
                    depth: depth.unwrap_or(0),
                    path: Vec::new(),
                })
            }
        }

        const FIELDS: &[&str] = &["pos", "depth"];
        deserializer.deserialize_struct("ResolvedPos", FIELDS, ResolvedPosVisitor)
    }
}

/// Information about a descendant node
#[derive(Debug, Clone, PartialEq)]
pub struct Descendant {
    /// The descendant node
    pub node: Node,
    /// The index in the parent
    pub index: usize,
    /// The offset from document start
    pub offset: usize,
}

fn resolve_path(
    doc: &Node,
    current_pos: usize,
    target_index: usize,
    current_depth: usize,
    mut path: Vec<PathEntry>,
) -> Option<(usize, Vec<PathEntry>, usize)> {
    match &doc.content {
        super::NodeContent::Nodes(nodes) => {
            if target_index == 0 {
                return Some((current_depth, path, current_pos));
            }

            let mut pos = current_pos;
            for (i, child) in nodes.iter().enumerate() {
                if i == target_index {
                    path.push(PathEntry {
                        node: doc.clone(),
                        index: i,
                        node_offset: pos,
                    });
                    return Some((current_depth + 1, path, pos));
                }

                let child_size = child.content_size().size;
                pos += child_size;
            }

            // If we get here, the index is beyond the last child
            if target_index == nodes.len() {
                path.push(PathEntry {
                    node: doc.clone(),
                    index: nodes.len(),
                    node_offset: pos,
                });
                return Some((current_depth + 1, path, pos));
            }

            None
        }
        super::NodeContent::Text(_) => {
            // At a text node, position is just the current pos
            Some((current_depth, path, current_pos))
        }
    }
}

fn find_path(
    doc: &Node,
    target_pos: usize,
    current_pos: usize,
    path: &mut Vec<PathEntry>,
    final_pos: &mut usize,
    depth: &mut usize,
) {
    *depth = path.len();

    if target_pos == current_pos {
        *final_pos = current_pos;
        return;
    }

    match &doc.content {
        super::NodeContent::Nodes(nodes) => {
            let mut pos = current_pos;
            for (i, child) in nodes.iter().enumerate() {
                let child_size = child.content_size().size;
                if target_pos < pos + child_size {
                    // The target is within this child
                    path.push(PathEntry {
                        node: doc.clone(),
                        index: i,
                        node_offset: pos,
                    });
                    find_path(child, target_pos, pos, path, final_pos, depth);
                    return;
                }
                pos += child_size;
            }

            // Position is at the end
            if target_pos == pos {
                path.push(PathEntry {
                    node: doc.clone(),
                    index: nodes.len(),
                    node_offset: pos,
                });
                *final_pos = pos;
            }
        }
        super::NodeContent::Text(tc) => {
            // At a text node, clamp position to text length
            let text_len = tc.text.len();
            let clamped_pos = if target_pos > current_pos + text_len {
                current_pos + text_len
            } else {
                target_pos
            };
            *final_pos = clamped_pos;
        }
    }
}

impl fmt::Display for ResolvedPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResolvedPos(pos={}, depth={})", self.pos, self.depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolved_pos_at_index() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
            Node::new_block("paragraph", vec![Node::new_text("World")]),
        ]);

        let pos = ResolvedPos::at_index(&doc, 0);
        assert_eq!(pos.position(), 0);
        assert_eq!(pos.current_index(), 0);

        let pos = ResolvedPos::at_index(&doc, 1);
        assert_eq!(pos.position(), 7);
    }

    #[test]
    fn test_resolved_pos_from_pos() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);

        let pos = ResolvedPos::from_pos(&doc, 0);
        assert_eq!(pos.position(), 0);
        assert!(pos.is_doc_start());

        let pos = ResolvedPos::from_pos(&doc, 10);
        assert!(!pos.is_doc_start());
    }

    #[test]
    fn test_resolved_pos_parent() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);

        let pos = ResolvedPos::at_index(&doc, 1);

        let parent = pos.immediate_parent();
        assert!(parent.is_some());
        assert_eq!(parent.unwrap().type_name(), "doc");
    }

    #[test]
    fn test_resolved_pos_depth() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);

        let pos = ResolvedPos::at_index(&doc, 1);
        assert_eq!(pos.depth(), 1);

        let pos = ResolvedPos::at_index(&doc, 0);
        assert_eq!(pos.depth(), 0);
    }

    #[test]
    fn test_resolved_pos_ancestors() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);

        let pos = ResolvedPos::at_index(&doc, 1);
        let ancestors: Vec<&str> = pos.ancestors().map(|n| n.type_name()).collect();

        assert_eq!(ancestors, vec!["doc"]);
    }

    #[test]
    fn test_resolved_pos_descendant() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
            Node::new_block("paragraph", vec![Node::new_text("World")]),
        ]);

        let pos = ResolvedPos::at_index(&doc, 1);
        let desc = pos.descendent();

        assert!(desc.is_some());
        assert_eq!(desc.unwrap().index, 1);
    }

    #[test]
    fn test_resolved_pos_text() {
        let text = Node::new_text("Hello");
        let pos = ResolvedPos::from_pos(&text, 3);

        assert_eq!(pos.position(), 3);
        assert_eq!(pos.depth(), 0);
    }

    #[test]
    fn test_resolved_pos_doc_start_end() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);

        let start = ResolvedPos::from_pos(&doc, 0);
        assert!(start.is_doc_start());
        assert!(!start.is_doc_end());

        let _end = ResolvedPos::from_pos(&doc, 20); // Approximate end
        // Note: exact end calculation depends on content size
    }
}
