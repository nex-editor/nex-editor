use crate::editor::editor_node::{EditorNode, EditorNodeProp, NodeType};

#[derive(Debug)]
pub struct ParagraphNodeStyle {
    text_alignment: ParagraphNodeAlignment,
    // px
    text_indent: usize,
}

#[derive(Debug)]
pub enum ParagraphNodeAlignment {
    Left,
    Center,
    Right,
}

pub fn create_paragraph_node() -> EditorNode {
    EditorNode::ParagraphNode {
        prop: EditorNodeProp {
            node_type: NodeType::ParagraphNode,
            first: None,
            last: None,
            next: None,
            prev: None,
            parent: None,
        },
        style: ParagraphNodeStyle {
            text_alignment: ParagraphNodeAlignment::Left,
            text_indent: 0
        }
    }
}