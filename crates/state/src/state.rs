//! Editor state
//!
//! The immutable state of the editor containing document, selection, and stored marks.

use super::{Selection, TextSelection, NodeSelection};
use model::{Node, Schema};
use model::node::Mark;
use serde::{Serialize, Deserialize};
use std::fmt;

/// Stored marks (marks applied but not yet used, e.g., bold before typing)
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StoredMarks {
    /// The stored marks
    marks: Vec<Mark>,
}

impl StoredMarks {
    /// Create empty stored marks
    pub fn empty() -> Self {
        Self { marks: Vec::new() }
    }

    /// Create with marks
    pub fn new(marks: Vec<Mark>) -> Self {
        Self { marks }
    }

    /// Get the marks
    pub fn marks(&self) -> &[Mark] {
        &self.marks
    }

    /// Add a mark
    pub fn add(&mut self, mark: Mark) {
        self.marks.push(mark);
    }

    /// Remove a mark by type
    pub fn remove(&mut self, mark_type: &str) {
        self.marks.retain(|m| m.type_name != mark_type);
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.marks.is_empty()
    }

    /// Clear all marks
    pub fn clear(&mut self) {
        self.marks.clear();
    }
}

impl From<Vec<Mark>> for StoredMarks {
    fn from(marks: Vec<Mark>) -> Self {
        Self { marks }
    }
}

/// The immutable editor state
///
/// This is the core state object that contains:
/// - The document tree
/// - The current selection
/// - Stored marks (for inline formatting)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditorState {
    /// The document
    doc: Node,
    /// The current selection
    selection: Selection,
    /// Stored marks (marks waiting to be applied)
    stored_marks: StoredMarks,
    /// The schema
    schema: Schema,
}

impl EditorState {
    /// Create a new editor state
    pub fn new(doc: Node, selection: Selection, schema: Schema) -> Self {
        Self {
            doc,
            selection,
            stored_marks: StoredMarks::empty(),
            schema,
        }
    }

    /// Create with stored marks
    pub fn with_marks(doc: Node, selection: Selection, stored_marks: StoredMarks, schema: Schema) -> Self {
        Self {
            doc,
            selection,
            stored_marks,
            schema,
        }
    }

    /// Create the initial state for a document
    pub fn from_doc(doc: Node) -> Self {
        let schema = Schema::default();
        let selection = Selection::text(TextSelection::at(&doc, 0));
        Self::new(doc, selection, schema)
    }

    /// Get the document
    pub fn doc(&self) -> &Node {
        &self.doc
    }

    /// Get the selection
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// Get the stored marks
    pub fn stored_marks(&self) -> &[Mark] {
        self.stored_marks.marks()
    }

    /// Get a reference to the schema
    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    /// Get the stored marks mutably
    pub fn stored_marks_mut(&mut self) -> &mut StoredMarks {
        &mut self.stored_marks
    }

    /// Get the text selection if applicable
    pub fn text_selection(&self) -> Option<&TextSelection> {
        self.selection.as_text()
    }

    /// Get the node selection if applicable
    pub fn node_selection(&self) -> Option<&NodeSelection> {
        self.selection.as_node()
    }

    /// Get the main selection range
    pub fn range(&self) -> Option<&super::SelectionRange> {
        match &self.selection {
            Selection::Text(t) => t.primary(),
            Selection::Node(_) => None,
        }
    }

    /// Check if selection is at document start
    pub fn is_at_start(&self) -> bool {
        self.selection.is_collapsed() && self.selection.head() == 0
    }

    /// Check if selection is at document end
    pub fn is_at_end(&self) -> bool {
        let doc_end = self.doc.content_size().content_size;
        self.selection.is_collapsed() && self.selection.head() == doc_end
    }

    /// Get the selection as text
    pub fn as_text(&self) -> String {
        if self.selection.is_collapsed() {
            return String::new();
        }
        // Simplified: extract text from range
        let from = self.selection.from();
        let to = self.selection.to();
        self.extract_text(from, to)
    }

    /// Extract text from a range
    fn extract_text(&self, from: usize, to: usize) -> String {
        self.doc
            .text_content()
            .chars()
            .skip(from)
            .take(to.saturating_sub(from))
            .collect()
    }

    /// Create a new state with updated document
    pub fn apply(&self, doc: Node) -> Self {
        Self {
            doc,
            selection: self.selection.clone(),
            stored_marks: self.stored_marks.clone(),
            schema: self.schema.clone(),
        }
    }

    /// Create a new state with updated selection
    pub fn with_selection(&self, selection: Selection) -> Self {
        Self {
            doc: self.doc.clone(),
            selection,
            stored_marks: self.stored_marks.clone(),
            schema: self.schema.clone(),
        }
    }

    /// Apply a selection to this state
    pub fn apply_selection(&self, selection: Selection) -> Self {
        self.with_selection(selection)
    }
}

impl fmt::Display for EditorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EditorState(doc={}, sel={})", self.doc.type_name(), self.selection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_state_new() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);
        let selection = Selection::text(TextSelection::at(&doc, 0));
        let schema = Schema::default();

        let state = EditorState::new(doc, selection, schema);

        assert_eq!(state.doc().type_name(), "doc");
        assert!(state.is_at_start());
    }

    #[test]
    fn test_editor_state_from_doc() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello World")]),
        ]);

        let state = EditorState::from_doc(doc);

        assert!(state.is_at_start());
        assert!(!state.is_at_end());
        assert_eq!(state.doc().text_content(), "Hello World");
    }

    #[test]
    fn test_editor_state_with_selection() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);

        let state = EditorState::from_doc(doc.clone());
        let new_state = state.with_selection(Selection::text(TextSelection::at(&doc, 5)));

        assert_ne!(state.selection().head(), new_state.selection().head());
        assert_eq!(new_state.selection().head(), 5);
    }

    #[test]
    fn test_stored_marks() {
        let _doc = Node::new_block("doc", vec![]);
        let mut marks = StoredMarks::empty();
        marks.add(Mark { type_name: "bold".to_string(), attrs: None });

        assert!(!marks.is_empty());
        assert_eq!(marks.marks().len(), 1);

        marks.remove("bold");
        assert!(marks.is_empty());
    }

    #[test]
    fn test_editor_state_with_marks() {
        let doc = Node::new_block("doc", vec![]);
        let marks = StoredMarks::new(vec![
            Mark { type_name: "italic".to_string(), attrs: None },
        ]);
        let selection = Selection::text(TextSelection::at(&doc, 0));
        let schema = Schema::default();

        let state = EditorState::with_marks(doc, selection, marks, schema);

        assert_eq!(state.stored_marks().len(), 1);
        assert_eq!(state.stored_marks()[0].type_name, "italic");
    }

    #[test]
    fn test_selection_types() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);

        let state = EditorState::from_doc(doc);

        // Initially should be text selection
        assert!(state.text_selection().is_some());
        assert!(state.node_selection().is_none());
    }
}
