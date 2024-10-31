use crate::editor::editor_node::EditorNode;
use crate::nodes::root_node::{create_root_node, ROOT_NODE_KEY};
use std::collections::HashMap;

#[derive(Debug)]
pub struct EditorState {
    pub node_map: HashMap<u32, EditorNode>,
}

impl EditorState {
    pub fn new() -> EditorState {
        EditorState {
            node_map: HashMap::new(),
        }
    }

    pub fn get_root_node(&mut self) -> &mut EditorNode {
        self.node_map.get_mut(&ROOT_NODE_KEY).unwrap()
    }

    pub fn insert_node(&mut self, key: u32, node: EditorNode) {
        self.node_map.insert(key, node);
    }

    pub fn print_node_map(&self) {
        for (key, node) in &self.node_map {
            println!("key: {}, node: {:?}", key, node);
        }
    }
}

pub fn create_empty_editor_state() -> EditorState {
    // root node
    let root_node = create_root_node();

    let mut editor_state = EditorState::new();
    editor_state.node_map.insert(ROOT_NODE_KEY, root_node);
    editor_state
}
