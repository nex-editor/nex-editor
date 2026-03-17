//! Base commands implementation
//!
//! Implements fundamental editor operations like join, split, delete, etc.

use state::{EditorState, Transaction};

/// Base commands trait
pub trait BaseCommands {
    /// Delete the selection
    fn delete_selection(&self, state: &EditorState) -> Transaction;

    /// Delete the selection and insert text
    fn type_text(&self, state: &EditorState, text: &str) -> Transaction;

    /// Delete backward from the current selection or cursor
    fn delete_backward(&self, state: &EditorState) -> Transaction;

    /// Delete forward from the current selection or cursor
    fn delete_forward(&self, state: &EditorState) -> Transaction;

    /// Join the selected block with its previous sibling
    fn join_backward(&self, state: &EditorState) -> Transaction;

    /// Join the selected block with its next sibling
    fn join_forward(&self, state: &EditorState) -> Transaction;

    /// Split the selected block at the cursor
    fn split_block(&self, state: &EditorState) -> Transaction;

    /// Move the cursor one character forward
    fn move_forward_char(&self, state: &EditorState) -> Transaction;

    /// Move the cursor one character backward
    fn move_backward_char(&self, state: &EditorState) -> Transaction;

    /// Move the cursor one line up
    fn move_up(&self, state: &EditorState) -> Transaction;

    /// Move the cursor one line down
    fn move_down(&self, state: &EditorState) -> Transaction;

    /// Select the whole document
    fn select_all(&self, state: &EditorState) -> Transaction;

    /// Toggle a mark (bold, italic, etc.)
    fn toggle_mark(&self, state: &EditorState, mark_type: &str) -> Transaction;

    /// Wrap selection in a block
    fn wrap_in(&self, state: &EditorState, block_type: &str) -> Transaction;

    /// Lift selection out of its parent
    fn lift(&self, state: &EditorState) -> Transaction;
}

/// Default implementation of base commands
#[derive(Debug, Clone, Default)]
pub struct BaseCommandsImpl;

impl BaseCommandsImpl {
    /// Create a new instance
    pub fn new() -> Self {
        Self
    }
}

impl BaseCommands for BaseCommandsImpl {
    fn delete_selection(&self, state: &EditorState) -> Transaction {
        let (from, to) = helpers::get_selection_range(state);
        let mut tr = Transaction::new(state.clone());
        if from != to {
            tr.delete(from, to).set_cursor(from);
        }
        tr
    }

    fn type_text(&self, state: &EditorState, text: &str) -> Transaction {
        let (from, to) = helpers::get_selection_range(state);
        let mut tr = Transaction::new(state.clone());
        tr.replace(from, to, transform::Slice::from_text(text))
            .set_cursor(from + text.len());
        tr
    }

    fn delete_backward(&self, state: &EditorState) -> Transaction {
        let (from, to) = helpers::get_selection_range(state);
        let mut tr = Transaction::new(state.clone());
        if from != to {
            tr.delete(from, to).set_cursor(from);
            return tr;
        }

        if from == 0 {
            return tr;
        }

        tr.delete(from - 1, from).set_cursor(from - 1);
        tr
    }

    fn delete_forward(&self, state: &EditorState) -> Transaction {
        let (from, to) = helpers::get_selection_range(state);
        let mut tr = Transaction::new(state.clone());
        if from != to {
            tr.delete(from, to).set_cursor(from);
            return tr;
        }

        let text_len = state.doc().plain_text_len();
        if from >= text_len {
            return tr;
        }

        tr.delete(from, from + 1).set_cursor(from);
        tr
    }

    fn join_backward(&self, state: &EditorState) -> Transaction {
        let pos = helpers::get_cursor(state);
        let mut tr = Transaction::new(state.clone());
        tr.add_structural_step(transform::JoinBackwardStep::new(pos))
            .set_cursor(pos);
        tr
    }

    fn join_forward(&self, state: &EditorState) -> Transaction {
        Transaction::new(state.clone())
    }

    fn split_block(&self, state: &EditorState) -> Transaction {
        let pos = helpers::get_cursor(state);
        let mut tr = Transaction::new(state.clone());
        tr.add_structural_step(transform::SplitBlockStep::new(pos))
            .set_cursor(pos);
        tr
    }

    fn move_forward_char(&self, state: &EditorState) -> Transaction {
        let pos = helpers::get_cursor(state);
        let next = (pos + 1).min(state.doc().plain_text_len());
        let mut tr = Transaction::new(state.clone());
        tr.set_cursor(next);
        tr
    }

    fn move_backward_char(&self, state: &EditorState) -> Transaction {
        let pos = helpers::get_cursor(state);
        let mut tr = Transaction::new(state.clone());
        tr.set_cursor(pos.saturating_sub(1));
        tr
    }

    fn move_up(&self, state: &EditorState) -> Transaction {
        Transaction::new(state.clone())
    }

    fn move_down(&self, state: &EditorState) -> Transaction {
        Transaction::new(state.clone())
    }

    fn select_all(&self, state: &EditorState) -> Transaction {
        let mut tr = Transaction::new(state.clone());
        tr.set_selection_range(0, state.doc().plain_text_len());
        tr
    }

    fn toggle_mark(&self, state: &EditorState, mark_type: &str) -> Transaction {
        let mut tr = Transaction::new(state.clone());
        let (from, to) = helpers::get_selection_range(state);
        let has_mark = state.stored_marks().iter().any(|mark| mark.type_name == mark_type);
        if has_mark {
            tr.remove_mark(from, to, mark_type);
        } else {
            let pos = helpers::get_cursor(state);
            tr.add_mark(from, to.max(pos), model::node::Mark {
                type_name: mark_type.to_string(),
                attrs: None,
            });
        }
        tr
    }

    fn wrap_in(&self, state: &EditorState, _block_type: &str) -> Transaction {
        Transaction::new(state.clone())
    }

    fn lift(&self, state: &EditorState) -> Transaction {
        Transaction::new(state.clone())
    }
}

/// Helper functions for command implementations
pub mod helpers {
    use state::{EditorState, Transaction};
    use model::ResolvedPos;

    /// Get the selection range as (from, to)
    pub fn get_selection_range(state: &EditorState) -> (usize, usize) {
        let sel = state.selection();
        (sel.from(), sel.to())
    }

    /// Get the cursor position (collapsed selection)
    pub fn get_cursor(state: &EditorState) -> usize {
        let sel = state.selection();
        if sel.is_collapsed() {
            sel.head()
        } else {
            sel.from()
        }
    }

    /// Get the resolved cursor position
    pub fn get_resolved_cursor(state: &EditorState) -> Option<ResolvedPos> {
        let sel = state.selection().as_text()?;
        sel.resolved_head().cloned()
    }

    /// Check if cursor is at document start
    pub fn is_at_doc_start(state: &EditorState) -> bool {
        get_cursor(state) == 0
    }

    /// Check if cursor is at document end
    pub fn is_at_doc_end(state: &EditorState) -> bool {
        let doc_end = state.doc().plain_text_len();
        get_cursor(state) >= doc_end
    }

    /// Create a transaction to delete a range
    pub fn delete_range(state: &EditorState, from: usize, to: usize) -> Transaction {
        let mut tr = Transaction::new(state.clone());
        tr.delete(from, to).set_cursor(from);
        tr
    }

    /// Create a transaction to insert text at cursor
    pub fn insert_at_cursor(state: &EditorState, text: &str, cursor_offset: usize) -> Transaction {
        let pos = get_cursor(state);
        let mut tr = Transaction::new(state.clone());
        tr.insert_text(pos, text)
            .set_cursor(pos + text.len() - cursor_offset);
        tr
    }
}

/// Command execution helpers
pub mod executor {
    use state::{EditorState, Transaction};

    /// Apply a command and return the new state
    pub fn apply_command<F>(state: &EditorState, make_transaction: F) -> EditorState
    where
        F: FnOnce() -> Transaction,
    {
        let tr = make_transaction();
        tr.commit().unwrap_or_else(|_| state.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use state::EditorState;
    use model::Node;

    #[test]
    fn test_base_commands_impl() {
        let cmd = BaseCommandsImpl::new();
        let state = EditorState::from_doc(Node::new_block("doc", vec![]));

        let tr = cmd.delete_selection(&state);
        assert_eq!(tr.step_count(), 0);
    }

    #[test]
    fn test_type_text() {
        let cmd = BaseCommandsImpl::new();
        let state = EditorState::from_doc(Node::new_block(
            "doc",
            vec![Node::new_block("paragraph", vec![Node::new_text("")])],
        ));

        let tr = cmd.type_text(&state, "Hello");
        assert_eq!(tr.step_count(), 1);
    }

    #[test]
    fn test_toggle_mark() {
        let cmd = BaseCommandsImpl::new();
        let state = EditorState::from_doc(Node::new_block("doc", vec![]));

        let tr = cmd.toggle_mark(&state, "bold");
        assert!(tr.stored_marks.is_some());
    }

    #[test]
    fn test_toggle_mark_with_selection() {
        let cmd = BaseCommandsImpl::new();
        let doc = Node::from_paragraph_texts(vec!["Hello".to_string()]);
        let state = EditorState::from_doc(doc.clone())
            .with_selection(state::Selection::text(state::TextSelection::create(&doc, 0, 5)));

        let new_state = executor::apply_command(&state, || cmd.toggle_mark(&state, "bold"));
        let paragraph = &new_state.doc().children().unwrap()[0];
        let text = &paragraph.children().unwrap()[0];
        match &text.content {
            model::NodeContent::Text(tc) => assert_eq!(tc.marks[0].type_name, "bold"),
            model::NodeContent::Nodes(_) => panic!("expected text node"),
        }
    }

    #[test]
    fn test_split_block() {
        let cmd = BaseCommandsImpl::new();
        let doc = model::Node::from_paragraph_texts(vec!["HelloWorld".to_string()]);
        let state = EditorState::from_doc(doc).with_selection(state::Selection::text(state::TextSelection::at(
            &model::Node::from_paragraph_texts(vec!["HelloWorld".to_string()]),
            5,
        )));

        let new_state = executor::apply_command(&state, || cmd.split_block(&state));
        assert_eq!(new_state.doc().paragraph_texts(), vec!["Hello".to_string(), "World".to_string()]);
    }

    #[test]
    fn test_join_backward() {
        let cmd = BaseCommandsImpl::new();
        let doc = model::Node::from_paragraph_texts(vec!["Hello".to_string(), "World".to_string()]);
        let state = EditorState::from_doc(doc.clone()).with_selection(state::Selection::text(state::TextSelection::at(
            &doc,
            6,
        )));

        let new_state = executor::apply_command(&state, || cmd.join_backward(&state));
        assert_eq!(new_state.doc().paragraph_texts(), vec!["HelloWorld".to_string()]);
    }

    #[test]
    fn test_helper_functions() {
        let doc = Node::new_block("doc", vec![
            Node::new_block("paragraph", vec![Node::new_text("Hello World")]),
        ]);
        let state = EditorState::from_doc(doc);

        assert!(helpers::is_at_doc_start(&state));

        let cursor = helpers::get_cursor(&state);
        assert_eq!(cursor, 0);

        let range = helpers::get_selection_range(&state);
        assert_eq!(range, (0, 0));
    }

    #[test]
    fn test_executor() {
        let doc = Node::new_block("doc", vec![]);
        let state = EditorState::from_doc(doc);

        let new_state = executor::apply_command(&state, || {
            BaseCommandsImpl::new().type_text(&state, "Hello")
        });

        assert_eq!(new_state.doc().plain_text(), "Hello");
    }

    #[test]
    fn test_delete_backward() {
        let cmd = BaseCommandsImpl::new();
        let doc = Node::from_paragraph_texts(vec!["Hello".to_string()]);
        let state = EditorState::from_doc(doc.clone())
            .with_selection(state::Selection::text(state::TextSelection::at(&doc, 5)));

        let new_state = executor::apply_command(&state, || cmd.delete_backward(&state));
        assert_eq!(new_state.doc().plain_text(), "Hell");
        assert_eq!(new_state.selection().head(), 4);
    }

    #[test]
    fn test_delete_forward() {
        let cmd = BaseCommandsImpl::new();
        let doc = Node::from_paragraph_texts(vec!["Hello".to_string()]);
        let state = EditorState::from_doc(doc.clone())
            .with_selection(state::Selection::text(state::TextSelection::at(&doc, 1)));

        let new_state = executor::apply_command(&state, || cmd.delete_forward(&state));
        assert_eq!(new_state.doc().plain_text(), "Hllo");
        assert_eq!(new_state.selection().head(), 1);
    }

    #[test]
    fn test_select_all() {
        let cmd = BaseCommandsImpl::new();
        let doc = Node::from_paragraph_texts(vec!["Hello".to_string(), "World".to_string()]);
        let state = EditorState::from_doc(doc);

        let new_state = executor::apply_command(&state, || cmd.select_all(&state));
        assert_eq!(new_state.selection().anchor(), 0);
        assert_eq!(new_state.selection().head(), 11);
    }
}
