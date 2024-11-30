use crate::editor::editor_node::EditorNode;
use crate::editor::editor_state::{get_node, EditorState};
use crate::nodes::NodeStyle;

// paragraph node + text node
pub fn paragraph_layout(
    editor_state: &mut EditorState,
    paragraph_node: &EditorNode,
    width: f32,
    height: f32,
) {
    let text_node_key = paragraph_node.get_first_node_key();
    let node_style = paragraph_node.get_style();

    if let NodeStyle::Paragraph(paragraph_node_style) = node_style {
        if let Some(text_node_key) = get_node(editor_state, text_node_key) {
            println!("paragraph node has text node")
        } else {
            println!("paragraph node has no text node")
        }
    } else {
        panic!("node is not paragraph")
    }
}