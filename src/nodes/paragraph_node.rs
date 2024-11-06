use crate::editor::editor_node::{EditorNode, EditorNodeProp, NodeTag, NodeType};
use crate::editor::editor_state::EditorState;
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
    let last = root_node.get_last();
    match last {
        Some(last) => {
            todo!("TODO")
        }
        None => {
            let paragraph_node = create_paragraph_node();
            let paragraph_node_key = generate_node_key();

            root_node.set_first(Some(paragraph_node_key));
            root_node.set_last(Some(paragraph_node_key));
            editor_state.insert_node(paragraph_node_key, paragraph_node);
        }
    }
}
