use crate::editor::editor_node::{EditorNode, NodeKey};
use crate::nodes::root_node::{create_root_node, ROOT_NODE_KEY};
use std::collections::HashMap;
use crate::nodes::paragraph_node::create_paragraph_node;
use crate::nodes::text_node::create_text_node;
use crate::page::page::PageName;
use crate::selection::selection::{create_empty_editor_selection, EditorSelection};
use crate::utils::utils::generate_node_key;

pub type NodeMap = HashMap<u32, EditorNode>;

#[derive(Debug)]
pub struct EditorState {
    pub node_map: NodeMap,
    pub selection: EditorSelection,
    pub page: PageName,
    pub scale: f32,
}

impl EditorState {
    pub fn new() -> EditorState {
        EditorState {
            node_map: HashMap::new(),
            selection: create_empty_editor_selection(),
            page: PageName::A4,
            scale: 1.0,
        }
    }

    pub fn get_node(&mut self, node_key: u32) -> Option<&mut EditorNode> {
        self.node_map.get_mut(&node_key)
    }

    pub fn append_node(&mut self, parent_key: NodeKey, node_key: NodeKey, mut child: EditorNode) {
        let parent_node = self.node_map.get_mut(&parent_key.unwrap()).unwrap();
        let last_node_key = parent_node.get_last_node_key();

        if let Some(last_node_key) = last_node_key {
            let last_node = self.node_map.get_mut(&last_node_key).unwrap();
            last_node.set_next_node_key(node_key);

            child.set_parent_node_key(parent_key);
            child.set_prev_node_key(Some(last_node_key));

            self.node_map.get_mut(&parent_key.unwrap()).unwrap().set_last_node_key(node_key);

        } else {
            parent_node.set_first_node_key(node_key);
            parent_node.set_last_node_key(node_key);

            child.set_parent_node_key(parent_key);
        }

        self.node_map.insert(node_key.unwrap(), child);
    }

    pub fn append_paragraph_node(&mut self) -> NodeKey {
        let paragraph_node = create_paragraph_node();
        let node_key = generate_node_key();
        self.append_node(Some(ROOT_NODE_KEY), Some(node_key), paragraph_node);
        Some(node_key)
    }

    pub fn append_text_node(&mut self, paragraph_node_key: u32, char: char) -> NodeKey  {
        let paragraph_node = self.node_map.get_mut(&paragraph_node_key);
        match paragraph_node {
            None => {
                eprintln!("append text node failed, paragraph node not found");
                None
            }
            Some(_) => {
                let text_node = create_text_node(char);
                let node_key = generate_node_key();
                self.append_node(Some(paragraph_node_key), Some(node_key), text_node);
                Some(node_key)
            }
        }
    }

    pub fn print_node_map(&self) {
        let mut keys = self.node_map.keys().copied().collect::<Vec<u32>>();

        keys.sort_unstable();

        for key in keys {
            println!("key: {}, node: {:?}", key, self.node_map.get(&key).unwrap());
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
