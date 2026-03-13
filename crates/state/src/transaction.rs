//! Transaction handling
//!
//! Builds and applies document changes through steps.

use super::{EditorState, Selection, TextSelection, StoredMarks};
use transform::{Step, StepMap, Mapping, ReplaceStep, SetMarksStep, Slice, Invertible};
use transform::step::StepError;
use model::Node;
use model::node::Mark;
use serde::{Serialize, Deserialize};
use std::fmt;

/// A transaction that produces a new state
///
/// Transactions are the main way to modify the editor state.
/// They accumulate steps and can be reversed (undone).
pub struct Transaction {
    /// The original state
    pub before: EditorState,
    /// The accumulated steps
    pub steps: Vec<StepWrapper>,
    /// The accumulated mapping
    pub mapping: Mapping,
    /// The updated selection (before being applied to state)
    pub selection: Option<Selection>,
    /// Updated stored marks
    pub stored_marks: Option<StoredMarks>,
    /// Whether this is a redo
    pub is_redo: bool,
    /// User-provided metadata
    pub meta: Option<TransactionMeta>,
}

/// A step wrapper for storage
pub struct StepWrapper {
    /// The step data
    pub step: Box<dyn Step>,
    /// The step map for this step
    pub map: StepMap,
    /// Inverted step for undo
    pub inverted: Option<Box<dyn Step>>,
}

impl fmt::Debug for StepWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StepWrapper")
            .field("map", &self.map)
            .field("has_inverted", &self.inverted.is_some())
            .finish()
    }
}

impl fmt::Debug for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transaction")
            .field("before", &self.before)
            .field("steps", &self.steps)
            .field("mapping", &self.mapping)
            .field("selection", &self.selection)
            .field("stored_marks", &self.stored_marks)
            .field("is_redo", &self.is_redo)
            .field("meta", &self.meta)
            .finish()
    }
}

impl StepWrapper {
    /// Create a new wrapper
    pub fn new(step: Box<dyn Step>, map: StepMap, inverted: Option<Box<dyn Step>>) -> Self {
        Self { step, map, inverted }
    }
}

/// Transaction metadata
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TransactionMeta {
    /// Description of the transaction
    pub description: Option<String>,
    /// Whether this was a user action
    pub user_action: bool,
    /// Timestamp
    pub timestamp: Option<u64>,
}

impl Transaction {
    /// Create a new transaction from a state
    pub fn new(state: EditorState) -> Self {
        Self {
            before: state,
            steps: Vec::new(),
            mapping: Mapping::new(),
            selection: None,
            stored_marks: None,
            is_redo: false,
            meta: None,
        }
    }

    /// Get the original state
    pub fn before(&self) -> &EditorState {
        &self.before
    }

    /// Get the number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get the mapping
    pub fn mapping(&self) -> &Mapping {
        &self.mapping
    }

    /// Get the steps
    pub fn steps(&self) -> &[StepWrapper] {
        &self.steps
    }

    /// Insert text at a position
    pub fn insert_text(&mut self, pos: usize, text: &str) -> &mut Self {
        let slice = Slice::from_text(text);
        self.replace(pos, pos, slice)
    }

    /// Delete text in a range
    pub fn delete(&mut self, from: usize, to: usize) -> &mut Self {
        self.replace(from, to, Slice::empty())
    }

    /// Replace a range with a slice
    pub fn replace(&mut self, from: usize, to: usize, slice: Slice) -> &mut Self {
        let step = ReplaceStep::new(from, to, slice);
        self.add_step(step)
    }

    /// Add a step to the transaction
    pub fn add_step<S: Step + Invertible + 'static>(&mut self, step: S) -> &mut Self {
        let doc = self.current_doc().unwrap_or_else(|_| self.before.doc().clone());

        // Apply the step
        match step.apply(&doc) {
            Ok(result) => {
                // Create the step map
                let step_map = StepMap::new(result.from, result.to, result.inserted_size());

                // Create the inverted step
                let inverted = step.invert(&doc);

                // Wrap and add
                self.steps.push(StepWrapper::new(
                    Box::new(step),
                    step_map.clone(),
                    Some(Box::new(inverted)),
                ));

                // Add to mapping
                self.mapping.add_map(step_map);
            }
            Err(e) => {
                // Log error but continue
                eprintln!("Step failed: {:?}", e);
            }
        }

        self
    }

    /// Add a structural step that currently has no inversion support.
    pub fn add_structural_step<S: Step + 'static>(&mut self, step: S) -> &mut Self {
        let doc = self.current_doc().unwrap_or_else(|_| self.before.doc().clone());

        match step.apply(&doc) {
            Ok(result) => {
                let step_map = StepMap::new(result.from, result.to, result.inserted_size());
                self.steps.push(StepWrapper::new(Box::new(step), step_map.clone(), None));
                self.mapping.add_map(step_map);
            }
            Err(e) => {
                eprintln!("Step failed: {:?}", e);
            }
        }

        self
    }

    fn current_doc(&self) -> Result<Node, StepError> {
        let mut doc = self.before.doc().clone();
        for wrapper in &self.steps {
            doc = wrapper.step.apply(&doc)?.doc;
        }
        Ok(doc)
    }

    /// Set the selection
    pub fn set_selection(&mut self, selection: Selection) -> &mut Self {
        self.selection = Some(selection);
        self
    }

    /// Set selection to a cursor position
    pub fn set_cursor(&mut self, pos: usize) -> &mut Self {
        self.selection = Some(Selection::text(TextSelection::at(self.before.doc(), pos)));
        self
    }

    /// Set selection to a range
    pub fn set_selection_range(&mut self, anchor: usize, head: usize) -> &mut Self {
        self.selection = Some(Selection::text(TextSelection::create(self.before.doc(), anchor, head)));
        self
    }

    /// Add a mark
    pub fn add_mark(&mut self, from: usize, to: usize, mark: Mark) -> &mut Self {
        if from == to {
            self.stored_marks.get_or_insert_with(StoredMarks::empty);
            if let Some(marks) = self.stored_marks.as_mut() {
                marks.add(mark);
            }
        } else {
            self.add_structural_step(SetMarksStep::add_mark(from, to, mark));
        }
        self
    }

    /// Remove a mark
    pub fn remove_mark(&mut self, from: usize, to: usize, mark_type: &str) -> &mut Self {
        if from == to {
            self.stored_marks.get_or_insert_with(StoredMarks::empty);
            if let Some(marks) = self.stored_marks.as_mut() {
                marks.remove(mark_type);
            }
        } else {
            self.add_structural_step(SetMarksStep::remove_mark(from, to, mark_type));
        }
        self
    }

    /// Clear stored marks
    pub fn clear_stored_marks(&mut self) -> &mut Self {
        self.stored_marks = Some(StoredMarks::empty());
        self
    }

    /// Set metadata
    pub fn with_meta(&mut self, meta: TransactionMeta) -> &mut Self {
        self.meta = Some(meta);
        self
    }

    /// Set description
    pub fn with_description(&mut self, desc: impl Into<String>) -> &mut Self {
        let meta = self.meta.take().unwrap_or_default();
        self.meta = Some(TransactionMeta {
            description: Some(desc.into()),
            ..meta
        });
        self
    }

    /// Mark as user action
    pub fn user_action(&mut self) -> &mut Self {
        let meta = self.meta.take().unwrap_or_default();
        self.meta = Some(TransactionMeta {
            user_action: true,
            ..meta
        });
        self
    }

    /// Apply the transaction to get a new state
    pub fn commit(&self) -> Result<EditorState, StepError> {
        let doc = self.current_doc()?;

        // Build the new selection
        let selection = if let Some(sel) = &self.selection {
            sel.resolve(&doc)
        } else {
            let old_sel = self.before.selection();
            match old_sel {
                Selection::Text(text_sel) => {
                    let anchor = self.mapping.map(text_sel.anchor()).pos;
                    let head = self.mapping.map(text_sel.head()).pos;
                    Selection::text(TextSelection::create(&doc, anchor, head))
                }
                Selection::Node(node_sel) => {
                    let pos = self.mapping.map(node_sel.pos()).pos;
                    Selection::node(super::NodeSelection::new(pos, node_sel.depth()))
                }
            }
        };

        // Build the new stored marks
        let stored_marks = self.stored_marks.clone()
            .unwrap_or_else(|| self.before.stored_marks().to_vec().into());

        Ok(EditorState::with_marks(
            doc,
            selection,
            stored_marks,
            self.before.schema().clone(),
        ))
    }

    /// Get the inverted transaction (for undo)
    pub fn invert(&self) -> Transaction {
        Transaction::new(self.before.clone())
    }
}

/// Builder for common transactions
#[derive(Debug, Clone)]
pub struct TransactionBuilder;

impl TransactionBuilder {
    /// Create a transaction from a state
    pub fn from(state: &EditorState) -> Transaction {
        Transaction::new(state.clone())
    }

    /// Create an insert transaction
    pub fn insert(state: &EditorState, pos: usize, text: &str) -> Transaction {
        let mut tr = Transaction::new(state.clone());
        tr.insert_text(pos, text);
        tr.user_action();
        tr
    }

    /// Create a delete transaction
    pub fn delete(state: &EditorState, from: usize, to: usize) -> Transaction {
        let mut tr = Transaction::new(state.clone());
        tr.delete(from, to);
        tr.user_action();
        tr
    }

    /// Create a replace transaction
    pub fn replace(state: &EditorState, from: usize, to: usize, slice: Slice) -> Transaction {
        let mut tr = Transaction::new(state.clone());
        tr.replace(from, to, slice);
        tr.user_action();
        tr
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transaction(steps={})", self.steps.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_new() {
        let doc = Node::new_block("doc", vec![]);
        let state = EditorState::from_doc(doc);

        let tr = Transaction::new(state.clone());

        assert_eq!(tr.step_count(), 0);
        assert!(tr.steps().is_empty());
    }

    #[test]
    fn test_transaction_insert_text() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);
        let state = EditorState::from_doc(doc);

        let mut tr = Transaction::new(state.clone());
        tr.insert_text(5, " World").set_cursor(11);

        assert_eq!(tr.step_count(), 1);
    }

    #[test]
    fn test_transaction_delete() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello World")]),
        ]);
        let state = EditorState::from_doc(doc);

        let mut tr = Transaction::new(state.clone());
        tr.delete(5, 11);

        assert_eq!(tr.step_count(), 1);
    }

    #[test]
    fn test_transaction_replace() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello World")]),
        ]);
        let state = EditorState::from_doc(doc);
        let slice = Slice::from_text("Rust");

        let mut tr = Transaction::new(state.clone());
        tr.replace(0, 5, slice);
        tr.with_description("Replace Hello with Rust");

        assert_eq!(tr.step_count(), 1);
        assert!(tr.meta.is_some());
        assert_eq!(tr.meta.as_ref().unwrap().description, Some("Replace Hello with Rust".to_string()));
    }

    #[test]
    fn test_transaction_set_selection() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);
        let state = EditorState::from_doc(doc);

        let mut tr = Transaction::new(state.clone());
        tr.set_selection_range(0, 5);

        assert!(tr.selection.is_some());
    }

    #[test]
    fn test_transaction_set_cursor() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);
        let state = EditorState::from_doc(doc);

        let mut tr = Transaction::new(state.clone());
        tr.set_cursor(5);

        assert!(tr.selection.is_some());
        if let Selection::Text(t) = tr.selection.as_ref().unwrap() {
            assert!(t.is_collapsed());
            assert_eq!(t.head(), 5);
        }
    }

    #[test]
    fn test_transaction_add_mark() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);
        let state = EditorState::from_doc(doc);

        let mut tr = Transaction::new(state.clone());
        tr.add_mark(0, 5, model::node::Mark {
            type_name: "bold".to_string(),
            attrs: None,
        });

        assert_eq!(tr.step_count(), 1);
        let new_state = tr.commit().expect("mark transaction should commit");
        let paragraph = &new_state.doc().children().unwrap()[0];
        let text = &paragraph.children().unwrap()[0];
        match &text.content {
            model::NodeContent::Text(tc) => assert_eq!(tc.marks[0].type_name, "bold"),
            model::NodeContent::Nodes(_) => panic!("expected text node"),
        }
    }

    #[test]
    fn test_transaction_commit() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);
        let state = EditorState::from_doc(doc);

        let mut tr = Transaction::new(state.clone());
        tr.insert_text(5, " World");
        tr.set_cursor(11);

        let new_state = tr.commit().expect("Transaction should commit");

        assert!(new_state.doc().content_size().content_size > 0);
    }

    #[test]
    fn test_transaction_builder() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello")]),
        ]);
        let state = EditorState::from_doc(doc);

        let tr = TransactionBuilder::insert(&state, 5, " World");

        assert_eq!(tr.step_count(), 1);
        assert!(tr.meta.is_some());
        assert!(tr.meta.as_ref().unwrap().user_action);
    }
}
