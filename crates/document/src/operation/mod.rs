//! Document operations for high-level editing.
//!
//! This module provides:
//! - `Operation`: A single document modification
//! - `OperationResult`: Result of applying an operation
//! - `OperationStack`: Stack for undo/redo support

use super::{Anchor, NodeContent, NodeId, NodeType, Selection, TextContent, TreeDoc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Types of document operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    // === Node Operations ===

    /// Insert a new node
    Insert {
        /// Parent node ID
        parent: NodeId,
        /// Position in parent's children
        index: usize,
        /// Node to insert
        node: NodeOperation,
    },

    /// Delete a node
    Delete {
        /// Node ID to delete
        node: NodeId,
    },

    /// Move a node
    Move {
        /// Node to move
        node: NodeId,
        /// New parent
        parent: NodeId,
        /// New position
        index: usize,
    },

    /// Set node type
    SetType {
        /// Node ID
        node: NodeId,
        /// New type
        node_type: NodeType,
    },

    // === Content Operations ===

    /// Insert text
    InsertText {
        /// Node ID
        node: NodeId,
        /// Position
        offset: usize,
        /// Text to insert
        text: String,
    },

    /// Delete text
    DeleteText {
        /// Node ID
        node: NodeId,
        /// Start position
        from: usize,
        /// End position
        to: usize,
    },

    // === Selection Operations ===

    /// Set selection
    SetSelection {
        /// New selection
        selection: Option<Selection>,
    },

    // === Mark Operations ===

    /// Add a mark/style
    AddMark {
        /// Node ID
        node: NodeId,
        /// Start position
        from: usize,
        /// End position
        to: usize,
        /// Mark key
        mark: String,
        /// Mark value
        value: serde_json::Value,
    },

    /// Remove a mark
    RemoveMark {
        /// Node ID
        node: NodeId,
        /// Start position
        from: usize,
        /// End position
        to: usize,
        /// Mark key
        mark: String,
    },
}

/// Serializable node operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeOperation {
    /// Node type
    pub node_type: NodeType,
    /// Content
    pub content: Option<String>,
    /// Attributes
    #[serde(default)]
    pub attrs: Vec<(String, serde_json::Value)>,
}

/// Result of applying an operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationResult {
    /// Updated selection
    pub selection: Option<Selection>,
    /// Nodes that were affected
    pub affected_nodes: Vec<NodeId>,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl OperationResult {
    pub fn success(affected: Vec<NodeId>) -> Self {
        Self {
            selection: None,
            affected_nodes: affected,
            success: true,
            error: None,
        }
    }

    pub fn with_selection(mut self, selection: Option<Selection>) -> Self {
        self.selection = selection;
        self
    }

    pub fn failed(message: &str) -> Self {
        Self {
            selection: None,
            affected_nodes: Vec::new(),
            success: false,
            error: Some(message.to_string()),
        }
    }
}

/// Stack of operations for undo/redo
#[derive(Debug, Clone)]
pub struct OperationStack {
    /// Stack of applied operations
    done: VecDeque<Operation>,
    /// Stack of undone operations
    undone: VecDeque<Operation>,
    /// Maximum history size
    max_size: usize,
}

impl OperationStack {
    /// Create a new stack
    pub fn new(max_size: usize) -> Self {
        Self {
            done: VecDeque::new(),
            undone: VecDeque::new(),
            max_size,
        }
    }

    /// Push an operation onto the stack
    pub fn push(&mut self, op: Operation) {
        self.done.push_back(op.clone());
        self.undone.clear();

        if self.done.len() > self.max_size {
            self.done.pop_front();
        }
    }

    /// Pop the last operation (for undo)
    pub fn pop(&mut self) -> Option<Operation> {
        self.done.pop_back()
    }

    /// Get the last operation without popping
    pub fn peek(&self) -> Option<&Operation> {
        self.done.back()
    }

    /// Push an undone operation back (for redo)
    pub fn push_undone(&mut self, op: Operation) {
        self.undone.push_back(op);
    }

    /// Pop an undone operation
    pub fn pop_undone(&mut self) -> Option<Operation> {
        self.undone.pop_back()
    }

    /// Check if can undo
    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    /// Check if can redo
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }

    /// Clear the stack
    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
    }
}

/// Transaction for applying multiple operations atomically
#[derive(Debug, Default)]
pub struct Transaction<'a> {
    doc: &'a mut TreeDoc,
    operations: Vec<Operation>,
    selection_before: Option<Selection>,
}

impl<'a> Transaction<'a> {
    /// Create a new transaction
    pub fn new(doc: &'a mut TreeDoc) -> Self {
        Self {
            doc,
            operations: Vec::new(),
            selection_before: None,
        }
    }

    /// Insert a paragraph
    pub fn insert_paragraph(
        &mut self,
        parent: NodeId,
        index: usize,
        text: &str,
    ) -> &mut Self {
        let content = NodeContent::Text(TextContent::with_text(text));
        self.operations.push(Operation::Insert {
            parent,
            index,
            node: NodeOperation {
                node_type: NodeType::Paragraph,
                content: Some(text.to_string()),
                attrs: Vec::new(),
            },
        });
        self
    }

    /// Insert text at a position
    pub fn insert_text(&mut self, node: NodeId, offset: usize, text: &str) -> &mut Self {
        self.operations.push(Operation::InsertText {
            node,
            offset,
            text: text.to_string(),
        });
        self
    }

    /// Delete text in a range
    pub fn delete_text(&mut self, node: NodeId, from: usize, to: usize) -> &mut Self {
        self.operations.push(Operation::DeleteText {
            node,
            from,
            to,
        });
        self
    }

    /// Delete a node
    pub fn delete(&mut self, node: NodeId) -> &mut Self {
        self.operations.push(Operation::Delete { node });
        self
    }

    /// Set selection
    pub fn set_selection(&mut self, selection: Option<Selection>) -> &mut Self {
        self.operations.push(Operation::SetSelection { selection });
        self
    }

    /// Add a mark
    pub fn add_mark(
        &mut self,
        node: NodeId,
        from: usize,
        to: usize,
        mark: &str,
        value: serde_json::Value,
    ) -> &mut Self {
        self.operations.push(Operation::AddMark {
            node,
            from,
            to,
            mark: mark.to_string(),
            value,
        });
        self
    }

    /// Apply all operations
    pub fn apply(mut self) -> OperationResult {
        let mut affected = Vec::new();

        for op in &self.operations {
            match op {
                Operation::InsertText { node, offset, text } => {
                    if let Some(node_data) = self.doc.get_mut(*node) {
                        if let NodeContent::Text(text_content) = &mut node_data.content {
                            text_content.text.insert(*offset, text).ok();
                            affected.push(*node);
                        }
                    }
                }
                Operation::DeleteText { node, from, to } => {
                    if let Some(node_data) = self.doc.get_mut(*node) {
                        if let NodeContent::Text(text_content) = &mut node_data.content {
                            text_content.text.delete(*from, *to - *from).ok();
                            affected.push(*node);
                        }
                    }
                }
                Operation::Delete { node } => {
                    self.doc.delete_node(*node);
                    affected.push(*node);
                }
                _ => {
                    // Other operations would be implemented here
                }
            }
        }

        OperationResult::success(affected)
    }
}

/// Extension trait for convenient operations
pub trait DocumentOperations {
    fn transaction(&mut self) -> Transaction;
}

impl DocumentOperations for TreeDoc {
    fn transaction(&mut self) -> Transaction {
        Transaction::new(self)
    }
}
