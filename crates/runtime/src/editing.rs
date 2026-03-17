use super::*;
use crate::commands::{RuntimeBaseCommands, base_commands};
use state::{Selection, TextSelection, Transaction};

impl EditorRuntime {
    pub(crate) fn set_selection(&mut self, anchor: FlatTextOffset, head: FlatTextOffset) {
        let text_len = self.state.doc().plain_text_len();
        let anchor = anchor.get().min(text_len);
        let head = head.get().min(text_len);
        let selection = Selection::text(TextSelection::create(self.state.doc(), anchor, head));
        self.state = self.state.with_selection(selection);
    }

    pub(crate) fn insert_text(&mut self, text: String) {
        let transaction = base_commands().type_text_transaction(&self.state, &text);
        self.apply_transaction(transaction);
    }

    pub(crate) fn backspace(&mut self) {
        let transaction = base_commands().delete_backward_transaction(&self.state);
        self.apply_transaction(transaction);
    }

    pub(crate) fn delete_forward(&mut self) {
        let transaction = base_commands().delete_forward_transaction(&self.state);
        self.apply_transaction(transaction);
    }

    pub(crate) fn move_caret_left(&mut self) {
        let selection = self.state.selection();
        if !selection.is_collapsed() {
            let next = FlatTextOffset::new(selection.from());
            self.set_selection(next, next);
            return;
        }
        let transaction = base_commands().move_left_transaction(&self.state);
        self.apply_transaction(transaction);
    }

    pub(crate) fn move_caret_right(&mut self) {
        let selection = self.state.selection();
        if !selection.is_collapsed() {
            let next = FlatTextOffset::new(selection.to());
            self.set_selection(next, next);
            return;
        }
        let transaction = base_commands().move_right_transaction(&self.state);
        self.apply_transaction(transaction);
    }

    pub(crate) fn select_all(&mut self) {
        let transaction = base_commands().select_all_transaction(&self.state);
        self.apply_transaction(transaction);
    }

    pub(crate) fn apply_transaction(&mut self, transaction: Transaction) {
        let previous_text = self.state.doc().plain_text();
        let Ok(next_state) = transaction.commit() else {
            return;
        };
        let next_text = next_state.doc().plain_text();
        if next_text != previous_text {
            self.revision += 1;
        }
        self.state = next_state;
    }
}
