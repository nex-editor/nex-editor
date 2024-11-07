use crate::editor::editor_node::{EditorNode, EditorNodeProp, NodeTag, NodeType};
use crate::editor::editor_state::EditorState;
use crate::nodes::root_node::ROOT_NODE_KEY;
use crate::utils::utils::generate_node_key;

pub fn create_paragraph_node() -> EditorNode {
    EditorNode::ParagraphNode {
        prop: EditorNodeProp {
            r#type: NodeType::ParagraphNode,
            first: None,
            last: None,
            next: None,
            prev: None,
            parent: None,
            tag: NodeTag::ParagraphNode,
            offset: 0,
        },
    }
}

pub fn append_paragraph_node(editor_state: &mut EditorState) {
    let root_node = editor_state.get_root_node();
    let last_key = root_node.get_last_key();
    match last_key {
        Some(last_key) => {
            let last_node = editor_state.get_node(last_key);

            if let Some(last_node) = last_node {
                let mut paragraph_node = create_paragraph_node();
                let paragraph_node_key = generate_node_key();

                paragraph_node.set_prev_key(Some(last_key));
                paragraph_node.set_next_key(None);
                paragraph_node.set_parent_key(Some(ROOT_NODE_KEY));

                last_node.set_next_key(Some(paragraph_node_key));
                last_node.set_next_key(Some(paragraph_node_key));

                editor_state.get_root_node().set_last_key(Some(paragraph_node_key));
                editor_state.insert_node(paragraph_node_key, paragraph_node);
            }
        }
        None => {
            let mut paragraph_node = create_paragraph_node();
            let paragraph_node_key = generate_node_key();
            root_node.set_first_key(Some(paragraph_node_key));
            root_node.set_last_key(Some(paragraph_node_key));
            paragraph_node.set_parent_key(Some(ROOT_NODE_KEY));
            editor_state.insert_node(paragraph_node_key, paragraph_node);
        }
    }
}
