use crate::editor::editor_node::{EditorNode, EditorNodeProp, NodeType};
use crate::nodes::{NodePadding, NodeStyle};

#[derive(Debug)]
pub struct ParagraphNodeStyle {
    pub text_alignment: ParagraphNodeAlignment,
    // px
    pub text_indent: usize,

    // padding
    pub padding: NodePadding,
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
        style: NodeStyle::Paragraph(ParagraphNodeStyle {
            text_alignment: ParagraphNodeAlignment::Left,
            text_indent: 0,
            padding: NodePadding {
                left: 0,
                right: 0,
                top: 0,
                bottom: 0,
            },
        }),
    }
}