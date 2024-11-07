pub mod editor_node;
pub mod editor_selection;
pub mod editor_state;

use crate::editor::editor_selection::{create_empty_editor_selection, EditorSelection};
use crate::editor::editor_state::{create_empty_editor_state, EditorState};
use crate::utils::command::{get_commit_id, get_tag};

#[derive(Debug)]
pub struct NexEditor {
    version: String,
    tag: String,
    pub state: EditorState,
    pub selection: EditorSelection,
}

impl NexEditor {
    pub fn new() -> NexEditor {
        let version = get_commit_id();
        let tag = get_tag();
        let state = create_empty_editor_state();
        let selection = create_empty_editor_selection();
        NexEditor {
            version,
            tag,
            state,
            selection,
        }
    }

    pub fn get_version(&self) -> String {
        self.version.clone()
    }

    pub fn get_tag(&self) -> String {
        self.tag.clone()
    }

    pub fn append_paragraph_node(&mut self) {
        self.state.append_paragraph_node();
    }
}
