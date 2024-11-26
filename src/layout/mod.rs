use crate::editor::editor_state::{get_node, EditorState};
use crate::nodes::root_node::ROOT_NODE_KEY;

pub fn layout_frame(editor_state: &mut EditorState) {
    let root_node = get_node(editor_state , Some(ROOT_NODE_KEY)).unwrap();
    let first_node_key = root_node.get_first_node_key();
    let first_node = get_node(editor_state, first_node_key);

    let page = editor_state.page.height;

    if let Some(first_node) = first_node {

    }
}