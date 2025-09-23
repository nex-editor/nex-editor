use crate::state::EditorState;

pub struct Editor {
    state: EditorState
}

impl Editor {
    pub fn new() -> Self {
        Self {
            state: EditorState::new()
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}