use crate::editor::editor_node::{EditorNode, EditorNodeProp, NodeTag, NodeType};

pub fn create_text_node(char: char) -> EditorNode {
    EditorNode::TextNode {
        prop: EditorNodeProp {
            r#type: NodeType::TextNode,
            tag: NodeTag::TextNode,
            first: None,
            last: None,
            next: None,
            prev: None,
            parent: None,
            offset: 0,
        },
        char,
    }
}
