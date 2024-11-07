use crate::editor::editor_node::{EditorNode, NodeKey};
use crate::nodes::root_node::{create_root_node, ROOT_NODE_KEY};
use std::collections::HashMap;
use crate::nodes::paragraph_node::create_paragraph_node;
use crate::utils::utils::generate_node_key;

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

    pub fn get_node(&mut self, node_key: u32) -> Option<&mut EditorNode> {
        self.node_map.get_mut(&node_key)
    }

    pub fn append_node(&mut self, parent_key: NodeKey, node_key: NodeKey, mut child: EditorNode) {
        let node = self.node_map.get_mut(&parent_key.unwrap()).unwrap();
        let last_node_key = node.get_last_node_key();

        if let Some(last_node_key) = last_node_key {
            let last_node = self.node_map.get_mut(&last_node_key).unwrap();
            last_node.set_next_node_key(node_key);

            child.set_parent_node_key(node_key);
            child.set_prev_node_key(Some(last_node_key));

        } else {
            node.set_first_node_key(node_key);
            node.set_last_node_key(node_key);

            child.set_parent_node_key(parent_key);
        }

        self.node_map.insert(node_key.unwrap(), child);
    }

    pub fn append_paragraph_node(&mut self) {
        let paragraph_node = create_paragraph_node();
        let node_key = generate_node_key();
        self.append_node(Some(ROOT_NODE_KEY), Some(node_key), paragraph_node);
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
