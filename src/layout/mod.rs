mod paragraph_layout;
mod measure;

use crate::editor::editor_node::NodeKey;
use crate::editor::editor_state::{get_node, EditorState};
use crate::nodes::root_node::ROOT_NODE_KEY;

pub fn layout_frame(editor_state: &mut EditorState) {
    let root_node = get_node(editor_state, Some(ROOT_NODE_KEY)).unwrap();
    let mut current_node = root_node;
    let mut current_node_key: NodeKey = Some(ROOT_NODE_KEY);

    let mut is_visited_node_map = std::collections::HashMap::new();

    loop {
        let first_node_key = current_node.get_first_node_key();
        let next_node_key = current_node.get_next_node_key();
        let parent_node_key = current_node.get_parent_node_key();

        // do Something
        println!("node_key: {:?}, node_type: {:?}", current_node_key.unwrap(), current_node);

        if let Some(first_node) = get_node(editor_state, first_node_key) {
            // avoid infinite loop
            if !is_visited_node_map.contains_key(&first_node_key) {
                current_node = first_node;
                current_node_key = first_node_key;
                is_visited_node_map.insert(current_node_key, true);
                continue;
            }
        }

        if let Some(next_node) = get_node(editor_state, next_node_key) {
            current_node = next_node;
            current_node_key = next_node_key;
            is_visited_node_map.insert(current_node_key, true);
            continue;
        }

        if let Some(parent_node) = get_node(editor_state, parent_node_key) {
            current_node = parent_node;
            current_node_key = parent_node_key;
            is_visited_node_map.insert(current_node_key, true);

            continue;
        }

        break;
    }
}