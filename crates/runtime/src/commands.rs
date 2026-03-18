use commands::{base::BaseCommandsImpl, BaseCommands};

pub(crate) fn base_commands() -> BaseCommandsImpl {
    BaseCommandsImpl::new()
}

pub(crate) trait RuntimeBaseCommands {
    fn type_text_transaction(&self, state: &state::EditorState, text: &str) -> state::Transaction;
    fn delete_backward_transaction(&self, state: &state::EditorState) -> state::Transaction;
    fn delete_forward_transaction(&self, state: &state::EditorState) -> state::Transaction;
    fn move_left_transaction(&self, state: &state::EditorState) -> state::Transaction;
    fn move_right_transaction(&self, state: &state::EditorState) -> state::Transaction;
    fn select_all_transaction(&self, state: &state::EditorState) -> state::Transaction;
}

impl RuntimeBaseCommands for BaseCommandsImpl {
    fn type_text_transaction(&self, state: &state::EditorState, text: &str) -> state::Transaction {
        self.type_text(state, text)
    }

    fn delete_backward_transaction(&self, state: &state::EditorState) -> state::Transaction {
        self.delete_backward(state)
    }

    fn delete_forward_transaction(&self, state: &state::EditorState) -> state::Transaction {
        self.delete_forward(state)
    }

    fn move_left_transaction(&self, state: &state::EditorState) -> state::Transaction {
        self.move_backward_char(state)
    }

    fn move_right_transaction(&self, state: &state::EditorState) -> state::Transaction {
        self.move_forward_char(state)
    }

    fn select_all_transaction(&self, state: &state::EditorState) -> state::Transaction {
        self.select_all(state)
    }
}
