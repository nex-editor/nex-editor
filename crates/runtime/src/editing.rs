use super::*;
use crate::commands::{base_commands, RuntimeBaseCommands};
use state::{Selection, TextSelection, Transaction};

impl EditorRuntime {
    pub(crate) fn start_composition(&mut self) {
        let selection = self.state.selection();
        self.composition = Some(CompositionState {
            from: selection.from(),
            to: selection.to(),
            text: String::new(),
        });
    }

    pub(crate) fn update_composition(&mut self, text: String) {
        if self.composition.is_none() {
            self.start_composition();
        }

        if let Some(composition) = self.composition.as_mut() {
            composition.text = text;
        }
    }

    pub(crate) fn commit_composition(&mut self, text: String) {
        let Some(composition) = self.composition.take() else {
            if !text.is_empty() {
                self.insert_text(text);
            }
            return;
        };

        let mut transaction = Transaction::new(self.state.clone());
        transaction
            .replace(
                composition.from,
                composition.to,
                transform::Slice::from_text(&text),
            )
            .set_cursor(composition.from + Node::char_len(&text));
        self.apply_transaction(transaction);
    }

    pub(crate) fn cancel_composition(&mut self) {
        self.composition = None;
    }

    pub(crate) fn set_selection(&mut self, anchor: FlatTextOffset, head: FlatTextOffset) {
        let text_len = self.state.doc().plain_text_len();
        let anchor = anchor.get().min(text_len);
        let head = head.get().min(text_len);
        let selection = Selection::text(TextSelection::create(self.state.doc(), anchor, head));
        self.state = self.state.with_selection(selection);
    }

    pub(crate) fn insert_text(&mut self, text: String) {
        self.cancel_composition();
        let transaction = base_commands().type_text_transaction(&self.state, &text);
        self.apply_transaction(transaction);
    }

    pub(crate) fn backspace(&mut self) {
        self.cancel_composition();
        let transaction = base_commands().delete_backward_transaction(&self.state);
        self.apply_transaction(transaction);
    }

    pub(crate) fn delete_forward(&mut self) {
        self.cancel_composition();
        let transaction = base_commands().delete_forward_transaction(&self.state);
        self.apply_transaction(transaction);
    }

    pub(crate) fn move_caret_left(&mut self) {
        self.cancel_composition();
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
        self.cancel_composition();
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
        self.cancel_composition();
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
