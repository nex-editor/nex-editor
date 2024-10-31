use crate::editor::editor_node::{EditorNode, EditorNodeProp, NodeTag, NodeType};

pub const ROOT_NODE_KEY: u32 = 0;

pub fn create_root_node() -> EditorNode {
    EditorNode::RootNode {
        prop: EditorNodeProp {
            r#type: NodeType::RootNode,
            first: None,
            last: None,
            next: None,
            prev: None,
            parent: None,
            tag: NodeTag::RootNode,
            offset: 0,
        },
    }
}
