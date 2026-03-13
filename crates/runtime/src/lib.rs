//! Headless plain-text runtime facade.

use model::Node;
use serde::{Deserialize, Serialize};
use state::{EditorState, Selection, TextSelection};

/// Serializable view consumed by UI shells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorSnapshot {
    pub text: String,
    pub selection_anchor: usize,
    pub selection_head: usize,
    pub revision: u64,
}

/// Plain-text commands accepted by the headless runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputCommand {
    SetText { text: String },
    SetSelection { anchor: usize, head: usize },
    InsertText { text: String },
    Backspace,
    DeleteForward,
    SelectAll,
}

/// Headless plain-text editor backed by the existing core state model.
#[derive(Debug, Clone)]
pub struct EditorRuntime {
    state: EditorState,
    revision: u64,
}

impl Default for EditorRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorRuntime {
    pub fn new() -> Self {
        Self {
            state: EditorState::from_doc(Node::from_paragraph_texts(vec![String::new()])),
            revision: 0,
        }
    }

    pub fn snapshot(&self) -> EditorSnapshot {
        EditorSnapshot {
            text: self.state.doc().text_content(),
            selection_anchor: self.state.selection().anchor(),
            selection_head: self.state.selection().head(),
            revision: self.revision,
        }
    }

    pub fn apply(&mut self, command: InputCommand) -> EditorSnapshot {
        match command {
            InputCommand::SetText { text } => self.set_text(text),
            InputCommand::SetSelection { anchor, head } => self.set_selection(anchor, head),
            InputCommand::InsertText { text } => self.insert_text(text),
            InputCommand::Backspace => self.backspace(),
            InputCommand::DeleteForward => self.delete_forward(),
            InputCommand::SelectAll => self.select_all(),
        }

        self.snapshot()
    }

    fn set_text(&mut self, text: String) {
        let changed = self.state.doc().text_content() != text;
        self.state = self.state_from_text(text.clone(), text.len(), text.len());
        if changed {
            self.revision += 1;
        }
    }

    fn set_selection(&mut self, anchor: usize, head: usize) {
        let text_len = self.state.doc().text_content().len();
        let anchor = anchor.min(text_len);
        let head = head.min(text_len);
        let selection = Selection::text(TextSelection::create(self.state.doc(), anchor, head));
        self.state = self.state.with_selection(selection);
    }

    fn insert_text(&mut self, text: String) {
        let selection = self.state.selection();
        let from = selection.from();
        let to = selection.to();
        let mut next = self.state.doc().text_content();
        next.replace_range(from..to, &text);
        let cursor = from + text.len();
        self.apply_text_edit(next, cursor, cursor);
    }

    fn backspace(&mut self) {
        let selection = self.state.selection();
        let from = selection.from();
        let to = selection.to();
        if from != to {
            let mut next = self.state.doc().text_content();
            next.replace_range(from..to, "");
            self.apply_text_edit(next, from, from);
            return;
        }

        if from == 0 {
            return;
        }

        let mut next = self.state.doc().text_content();
        next.replace_range((from - 1)..from, "");
        let cursor = from - 1;
        self.apply_text_edit(next, cursor, cursor);
    }

    fn delete_forward(&mut self) {
        let selection = self.state.selection();
        let from = selection.from();
        let to = selection.to();
        if from != to {
            let mut next = self.state.doc().text_content();
            next.replace_range(from..to, "");
            self.apply_text_edit(next, from, from);
            return;
        }

        let text_len = self.state.doc().text_content().len();
        if from >= text_len {
            return;
        }

        let mut next = self.state.doc().text_content();
        next.replace_range(from..(from + 1), "");
        self.apply_text_edit(next, from, from);
    }

    fn select_all(&mut self) {
        let text_len = self.state.doc().text_content().len();
        self.set_selection(0, text_len);
    }

    fn state_from_text(&self, text: String, anchor: usize, head: usize) -> EditorState {
        let doc = Node::from_paragraph_texts(vec![text]);
        let selection = Selection::text(TextSelection::create(&doc, anchor, head));
        EditorState::new(doc, selection, self.state.schema().clone())
    }

    fn apply_text_edit(&mut self, next_text: String, anchor: usize, head: usize) {
        let changed = self.state.doc().text_content() != next_text;
        if !changed {
            return;
        }

        self.state = self.state_from_text(next_text, anchor, head);
        self.revision += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_text_updates_snapshot() {
        let mut runtime = EditorRuntime::new();
        let snapshot = runtime.apply(InputCommand::SetText {
            text: "hello".to_string(),
        });

        assert_eq!(snapshot.text, "hello");
        assert_eq!(snapshot.selection_anchor, 5);
        assert_eq!(snapshot.selection_head, 5);
        assert_eq!(snapshot.revision, 1);
    }

    #[test]
    fn test_insert_text_replaces_selection() {
        let mut runtime = EditorRuntime::new();
        runtime.apply(InputCommand::SetText {
            text: "hello".to_string(),
        });
        runtime.apply(InputCommand::SetSelection { anchor: 1, head: 4 });
        let snapshot = runtime.apply(InputCommand::InsertText {
            text: "i".to_string(),
        });

        assert_eq!(snapshot.text, "hio");
        assert_eq!(snapshot.selection_anchor, 2);
        assert_eq!(snapshot.selection_head, 2);
    }

    #[test]
    fn test_backspace_deletes_previous_character() {
        let mut runtime = EditorRuntime::new();
        runtime.apply(InputCommand::SetText {
            text: "hello".to_string(),
        });
        runtime.apply(InputCommand::SetSelection { anchor: 5, head: 5 });

        let snapshot = runtime.apply(InputCommand::Backspace);
        assert_eq!(snapshot.text, "hell");
        assert_eq!(snapshot.selection_head, 4);
    }

    #[test]
    fn test_delete_forward_deletes_next_character() {
        let mut runtime = EditorRuntime::new();
        runtime.apply(InputCommand::SetText {
            text: "hello".to_string(),
        });
        runtime.apply(InputCommand::SetSelection { anchor: 1, head: 1 });

        let snapshot = runtime.apply(InputCommand::DeleteForward);
        assert_eq!(snapshot.text, "hllo");
        assert_eq!(snapshot.selection_head, 1);
    }

    #[test]
    fn test_set_selection_clamps() {
        let mut runtime = EditorRuntime::new();
        runtime.apply(InputCommand::SetText {
            text: "abc".to_string(),
        });

        let snapshot = runtime.apply(InputCommand::SetSelection { anchor: 10, head: 20 });
        assert_eq!(snapshot.selection_anchor, 3);
        assert_eq!(snapshot.selection_head, 3);
        assert_eq!(snapshot.revision, 1);
    }

    #[test]
    fn test_select_all() {
        let mut runtime = EditorRuntime::new();
        runtime.apply(InputCommand::SetText {
            text: "abc".to_string(),
        });

        let snapshot = runtime.apply(InputCommand::SelectAll);
        assert_eq!(snapshot.selection_anchor, 0);
        assert_eq!(snapshot.selection_head, 3);
        assert_eq!(snapshot.revision, 1);
    }

    #[test]
    fn test_revision_changes_only_on_edit() {
        let mut runtime = EditorRuntime::new();
        assert_eq!(runtime.snapshot().revision, 0);

        runtime.apply(InputCommand::SetSelection { anchor: 0, head: 0 });
        assert_eq!(runtime.snapshot().revision, 0);

        runtime.apply(InputCommand::InsertText {
            text: "a".to_string(),
        });
        assert_eq!(runtime.snapshot().revision, 1);

        runtime.apply(InputCommand::SelectAll);
        assert_eq!(runtime.snapshot().revision, 1);
    }
}
