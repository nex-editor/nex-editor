use crate::color::Color;
use crate::editor::editor_node::{EditorNode, EditorNodeProp, NodeType};
use crate::nodes::NodeStyle;

#[derive(Debug)]
pub struct TextNodeStyle {
    font_family: String,
    font_size: usize,
    font_weight: usize,
    color: Color,
    background_color: Color,
}

pub fn create_text_node(char: char) -> EditorNode {
    EditorNode::TextNode {
        prop: EditorNodeProp {
            node_type: NodeType::TextNode,
            first: None,
            last: None,
            next: None,
            prev: None,
            parent: None,
        },
        char,
        style: NodeStyle::Text(TextNodeStyle {
            font_family: String::from("monospace"),
            font_size: 12,
            font_weight: 400,
            color: Color::BLACK,
            background_color: Color::TRANSPARENT,
        }),
    }
}
