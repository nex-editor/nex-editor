use crate::editor::editor_state::{EditorState, NodeMap};
use crate::editor::NexEditor;
use crate::nodes::root_node::ROOT_NODE_KEY;

pub fn layout_frame(editor_state: &mut EditorState) {
    let root_node = editor_state.get_node(ROOT_NODE_KEY).unwrap();
}