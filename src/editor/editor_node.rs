use fontdb::Style;
use crate::nodes::paragraph_node::ParagraphNodeStyle;
use crate::nodes::text_node::TextNodeStyle;

pub type NodeKey = Option<u32>;

#[derive(Debug, PartialEq)]
pub enum NodeType {
    RootNode = 1,
    TextNode = 2,
    ParagraphNode = 3,
}

#[derive(Debug)]
pub struct EditorNodeProp {
    pub node_type: NodeType,
    pub first: NodeKey,
    pub last: NodeKey,
    pub next: NodeKey,
    pub prev: NodeKey,
    pub parent: NodeKey,
}

#[derive(Debug)]
pub enum EditorNode {
    RootNode {
        prop: EditorNodeProp,
    },
    TextNode {
        prop: EditorNodeProp,
        char: char,
        style: TextNodeStyle,
    },
    ParagraphNode {
        prop: EditorNodeProp,
        style: ParagraphNodeStyle,
    },
}

impl EditorNode {
    pub fn is_type(&self, node_type: NodeType) -> bool {
        match self {
            EditorNode::RootNode { prop } => prop.node_type == node_type,
            EditorNode::TextNode { prop, .. } => prop.node_type == node_type,
            EditorNode::ParagraphNode { prop, .. } => prop.node_type == node_type,
        }
    }

    pub fn set_first_node_key(&mut self, first: NodeKey) {
        match self {
            EditorNode::RootNode { prop } => prop.first = first,
            EditorNode::TextNode { prop, .. } => prop.first = first,
            EditorNode::ParagraphNode { prop, .. } => prop.first = first,
        }
    }

    pub fn get_first_node_key(&self) -> NodeKey {
        match self {
            EditorNode::RootNode { prop } => prop.first,
            EditorNode::TextNode { prop, .. } => prop.first,
            EditorNode::ParagraphNode { prop, .. } => prop.first,
        }
    }

    pub fn set_last_node_key(&mut self, last: NodeKey) {
        match self {
            EditorNode::RootNode { prop } => prop.last = last,
            EditorNode::TextNode { prop, .. } => prop.last = last,
            EditorNode::ParagraphNode { prop, .. } => prop.last = last,
        }
    }

    pub fn get_last_node_key(&self) -> NodeKey {
        match self {
            EditorNode::RootNode { prop } => prop.last,
            EditorNode::TextNode { prop, .. } => prop.last,
            EditorNode::ParagraphNode { prop, .. } => prop.last,
        }
    }

    pub fn set_prev_node_key(&mut self, prev: NodeKey) {
        match self {
            EditorNode::RootNode { prop } => prop.prev = prev,
            EditorNode::TextNode { prop, .. } => prop.prev = prev,
            EditorNode::ParagraphNode { prop, .. } => prop.prev = prev,
        }
    }

    pub fn get_prev_node_key(&self) -> NodeKey {
        match self {
            EditorNode::RootNode { prop } => prop.prev,
            EditorNode::TextNode { prop, .. } => prop.prev,
            EditorNode::ParagraphNode { prop, .. } => prop.prev,
        }
    }

    pub fn set_next_node_key(&mut self, next: NodeKey) {
        match self {
            EditorNode::RootNode { prop } => prop.next = next,
            EditorNode::TextNode { prop, .. } => prop.next = next,
            EditorNode::ParagraphNode { prop, .. } => prop.next = next,
        }
    }

    pub fn get_next_node_key(&self) -> NodeKey {
        match self {
            EditorNode::RootNode { prop } => prop.next,
            EditorNode::TextNode { prop, .. } => prop.next,
            EditorNode::ParagraphNode { prop, .. } => prop.next,
        }
    }

    pub fn set_parent_node_key(&mut self, parent: NodeKey) {
        match self {
            EditorNode::RootNode { prop } => prop.parent = parent,
            EditorNode::TextNode { prop, .. } => prop.parent = parent,
            EditorNode::ParagraphNode { prop, .. } => prop.parent = parent,
        }
    }

    pub fn get_parent_node_key(&self) -> NodeKey {
        match self {
            EditorNode::RootNode { prop } => prop.parent,
            EditorNode::TextNode { prop, .. } => prop.parent,
            EditorNode::ParagraphNode { prop, .. } => prop.parent,
        }
    }
    
    pub fn get_char(&self) -> char {
        match self {
            EditorNode::TextNode { prop: _, char, .. } => *char,
            _ => panic!("TextNode is required"),
        }
    }
}