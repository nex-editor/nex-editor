use crate::nodes::paragraph_node::ParagraphNodeStyle;
use crate::nodes::text_node::TextNodeStyle;

pub mod root_node;

pub mod text_node;

pub mod paragraph_node;

#[derive(Debug)]
pub enum NodeStyle {
    Paragraph(ParagraphNodeStyle),
    Text(TextNodeStyle),
}

#[derive(Debug)]
pub struct NodePadding {
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
}