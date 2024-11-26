use crate::editor::editor_node::{EditorNode, EditorNodeProp, NodeType};

pub const ROOT_NODE_KEY: u32 = 0;

pub fn create_root_node() -> EditorNode {
    EditorNode::RootNode {
        prop: EditorNodeProp {
            node_type: NodeType::RootNode,
            first: None,
            last: None,
            next: None,
            prev: None,
            parent: None,
        },
    }
}
