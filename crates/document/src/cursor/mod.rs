//! Cursor and selection management.
//!
//! This module provides:
//! - `Anchor`: A single position in the document
//! - `Selection`: A range between two anchors
//! - Utilities for working with selections

use super::{NodeId, TreeDoc};
use std::cmp::Ordering;

/// A single position in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Anchor {
    /// The node ID where this anchor is located
    pub node_id: NodeId,
    /// Character offset within the node
    pub offset: usize,
}

impl Anchor {
    /// Create a new anchor
    pub fn new(node_id: NodeId, offset: usize) -> Self {
        Self { node_id, offset }
    }

    /// Create an anchor at the start of a node
    pub fn at_start(node_id: NodeId) -> Self {
        Self { node_id, offset: 0 }
    }

    /// Create an anchor at the end of a node
    pub fn at_end(node_id: NodeId, length: usize) -> Self {
        Self { node_id, offset: length }
    }

    /// Get the offset
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get the node ID
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Compare two anchors
    pub fn cmp(&self, other: &Self, doc: &TreeDoc) -> Ordering {
        // First compare by depth (shallower = earlier)
        let depth_cmp = doc.depth(self.node_id).cmp(&doc.depth(other.node_id));
        if depth_cmp != Ordering::Equal {
            return depth_cmp;
        }

        // Then by node ID (smaller = earlier)
        self.node_id.cmp(&other.node_id)
    }
}

/// A selection range between two anchors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    /// The anchor point (start of selection)
    pub anchor: Anchor,
    /// The focus point (end of selection)
    pub focus: Anchor,
}

impl Selection {
    /// Create a caret (collapsed selection)
    pub fn caret(anchor: Anchor) -> Self {
        Self {
            anchor,
            focus: anchor,
        }
    }

    /// Create a selection range
    pub fn range(anchor: Anchor, focus: Anchor) -> Self {
        Self { anchor, focus }
    }

    /// Check if this is a caret (no selection)
    pub fn is_caret(&self) -> bool {
        self.anchor == self.focus
    }

    /// Get the start anchor (minimum position)
    pub fn start(&self) -> Anchor {
        self.anchor
    }

    /// Get the end anchor (maximum position)
    pub fn end(&self) -> Anchor {
        self.focus
    }

    /// Get the anchor at a specific side
    pub fn anchor(&self, side: SelectionSide) -> Anchor {
        match side {
            SelectionSide::Start => self.anchor,
            SelectionSide::End => self.focus,
        }
    }

    /// Get the minimum position
    pub fn min(&self) -> Anchor {
        if self.anchor <= self.focus {
            self.anchor
        } else {
            self.focus
        }
    }

    /// Get the maximum position
    pub fn max(&self) -> Anchor {
        if self.anchor >= self.focus {
            self.anchor
        } else {
            self.focus
        }
    }

    /// Get the length of the selection (in characters)
    pub fn len(&self, doc: &TreeDoc) -> usize {
        if self.is_caret() {
            return 0;
        }
        // Simplified - in reality would calculate across nodes
        self.focus.offset.saturating_sub(self.anchor.offset)
    }

    /// Check if an anchor is contained in this selection
    pub fn contains(&self, anchor: &Anchor) -> bool {
        *anchor >= self.min() && *anchor <= self.max()
    }

    /// Check if a node is contained in this selection
    pub fn contains_node(&self, node_id: NodeId) -> bool {
        self.anchor.node_id == node_id || self.focus.node_id == node_id
    }

    /// Expand the selection to include a node
    pub fn expand_to(&self, anchor: Anchor) -> Self {
        Self {
            anchor: if anchor < self.min() { anchor } else { self.anchor },
            focus: if anchor > self.max() { anchor } else { self.focus },
        }
    }

    /// Collapse the selection to an anchor
    pub fn collapse(&self, to: SelectionSide) -> Self {
        match to {
            SelectionSide::Start => Self::caret(self.anchor),
            SelectionSide::End => Self::caret(self.focus),
        }
    }

    /// Move the focus while keeping the anchor
    pub fn move_focus(&mut self, to: Anchor) {
        self.focus = to;
    }

    /// Move both anchor and focus
    pub fn move_both(&mut self, anchor: Anchor, focus: Anchor) {
        self.anchor = anchor;
        self.focus = focus;
    }
}

/// Which side of a selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionSide {
    Start,
    End,
}

/// Direction for cursor movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
    Up,
    Down,
}

/// Result of a cursor movement operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorMoveResult {
    /// The new cursor position
    pub cursor: Option<Anchor>,
    /// Whether the move was successful
    pub success: bool,
    /// Optional message (e.g., "at document start")
    pub message: Option<&'static str>,
}

impl CursorMoveResult {
    pub fn success(cursor: Anchor) -> Self {
        Self {
            cursor: Some(cursor),
            success: true,
            message: None,
        }
    }

    pub fn failed(message: &'static str) -> Self {
        Self {
            cursor: None,
            success: false,
            message: Some(message),
        }
    }
}

/// Cursor movement utilities
pub struct CursorNavigator<'a> {
    doc: &'a TreeDoc,
}

impl<'a> CursorNavigator<'a> {
    /// Create a new navigator
    pub fn new(doc: &'a TreeDoc) -> Self {
        Self { doc }
    }

    /// Move cursor forward by one character
    pub fn move_forward(&self, cursor: Anchor) -> CursorMoveResult {
        // Get the node content
        if let Some(node) = self.doc.get(cursor.node_id) {
            if let Some(text) = self.get_text_content(node) {
                if cursor.offset < text.len() {
                    return CursorMoveResult::success(Anchor::new(cursor.node_id, cursor.offset + 1));
                }
            }
        }

        // Try to move to next sibling
        if let Some(next) = self.doc.next_sibling(cursor.node_id) {
            if let Some(text) = self.get_text_content(next) {
                return CursorMoveResult::success(Anchor::at_start(next.id));
            }
        }

        // Try to move to parent's next sibling
        self.move_to_next_node(cursor)
    }

    /// Move cursor backward by one character
    pub fn move_backward(&self, cursor: Anchor) -> CursorMoveResult {
        if cursor.offset > 0 {
            return CursorMoveResult::success(Anchor::new(cursor.node_id, cursor.offset - 1));
        }

        // Try to move to previous sibling's end
        if let Some(prev) = self.doc.prev_sibling(cursor.node_id) {
            if let Some(text) = self.get_text_content(prev) {
                return CursorMoveResult::success(Anchor::at_end(prev.id, text.len()));
            }
        }

        // Try to move to parent's previous
        if let Some(parent_id) = self.doc.get(cursor.node_id).and_then(|n| n.parent) {
            return CursorMoveResult::success(Anchor::at_end(parent_id, 0));
        }

        CursorMoveResult::failed("already at document start")
    }

    /// Move to the next node
    fn move_to_next_node(&self, cursor: Anchor) -> CursorMoveResult {
        let mut current = cursor.node_id;

        // Go up until we find a parent with a next sibling
        loop {
            if let Some(parent_id) = self.doc.get(current).and_then(|n| n.parent) {
                if let Some(next_sibling) = self.get_next_sibling(parent_id, current) {
                    // Find the first text node in this subtree
                    if let Some(first_text) = self.find_first_text(next_sibling.id) {
                        return CursorMoveResult::success(Anchor::at_start(first_text.id));
                    }
                }
                current = parent_id;
            } else {
                return CursorMoveResult::failed("already at document end");
            }
        }
    }

    /// Get the next sibling after a given node
    fn get_next_sibling(&self, parent_id: NodeId, after: NodeId) -> Option<&super::Node> {
        let children = self.doc.children(parent_id)?;
        let mut found = false;
        for &child_id in children {
            if found {
                return self.doc.get(child_id);
            }
            if child_id == after {
                found = true;
            }
        }
        None
    }

    /// Find the first text node in a subtree
    fn find_first_text(&self, node_id: NodeId) -> Option<&super::Node> {
        let node = self.doc.get(node_id)?;
        if let Some(text) = self.get_text_content(node) {
            return Some(node);
        }
        for &child_id in &node.children {
            if let Some(found) = self.find_first_text(child_id) {
                return Some(found);
            }
        }
        None
    }

    /// Get text content from a node
    fn get_text_content<'b>(&self, node: &'b super::Node) -> Option<&'b str> {
        if let super::NodeContent::Text(text) = &node.content {
            Some(text.text.as_str())
        } else {
            None
        }
    }
}

/// Helper trait for anchor comparison
impl PartialOrd for Anchor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other, &TreeDoc::new()))
    }
}

impl Ord for Anchor {
    fn cmp(&self, other: &Self) -> Ordering {
        self.node_id.cmp(&other.node_id)
    }
}
