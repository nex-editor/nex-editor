use crate::editor::editor_node::{EditorNode, EditorNodeProp, NodeTag, NodeType};

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