//! nex-editor Document Module
//!
//! A tree-based document model with CRDT support using Loro.

use loro::LoroDoc;
use std::collections::HashMap;
use std::hash::Hash;

// ============================================
// Node Types
// ============================================

/// Unique identifier for a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    pub fn new(n: u64) -> Self { Self(n) }
    pub fn inner(&self) -> u64 { self.0 }
}

/// Types of nodes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    Document,
    Paragraph,
    Heading { level: u8 },
    List { ordered: bool },
    ListItem,
    Blockquote,
    CodeBlock { lang: Option<String> },
    Divider,
    Text,
    Link { url: Option<String>, title: Option<String> },
    Image { src: String, alt: Option<String> },
    Table,
    TableRow,
    TableCell,
}

impl NodeType {
    pub fn is_block(&self) -> bool {
        matches!(self, NodeType::Paragraph | NodeType::Heading { .. }
            | NodeType::List { .. } | NodeType::ListItem | NodeType::Blockquote
            | NodeType::CodeBlock { .. } | NodeType::Divider)
    }
    pub fn is_inline(&self) -> bool {
        matches!(self, NodeType::Text | NodeType::Link { .. } | NodeType::Image { .. })
    }
    pub fn is_leaf(&self) -> bool {
        matches!(self, NodeType::Text | NodeType::Image { .. })
    }
}

/// Content types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeContent {
    Empty,
    Text { text: String },
    Mixed { children: Vec<NodeId> },
}

/// Node metadata
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeMeta {
    pub collapsed: bool,
    pub deleted: bool,
    pub attrs: HashMap<String, serde_json::Value>,
}

/// Complete node structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub node_type: NodeType,
    pub content: NodeContent,
    pub meta: NodeMeta,
}

impl Node {
    pub fn new(id: NodeId, node_type: NodeType, content: NodeContent) -> Self {
        Self { id, parent: None, children: Vec::new(), node_type, content, meta: NodeMeta::default() }
    }
    pub fn is_leaf(&self) -> bool { self.node_type.is_leaf() || self.children.is_empty() }
}

// ============================================
// TreeDoc
// ============================================

/// Tree-based document
#[derive(Debug)]
pub struct TreeDoc {
    doc: LoroDoc,
    root: NodeId,
    nodes: HashMap<NodeId, Node>,
    id_counter: u64,
}

impl TreeDoc {
    pub fn new() -> Self {
        let doc = LoroDoc::new();
        let root = NodeId(0);
        let mut nodes = HashMap::new();
        nodes.insert(root, Node::new(root, NodeType::Document, NodeContent::Empty));
        Self { doc, root, nodes, id_counter: 1 }
    }

    pub fn loro_doc(&self) -> &LoroDoc { &self.doc }

    fn new_id(&mut self) -> NodeId {
        let id = self.id_counter;
        self.id_counter += 1;
        NodeId(id)
    }

    pub fn create_node(&mut self, node_type: NodeType, content: NodeContent) -> NodeId {
        let id = self.new_id();
        let node = Node::new(id, node_type, content);
        self.nodes.insert(id, node);
        id
    }

    pub fn append_node(&mut self, parent_id: NodeId, node_type: NodeType, content: NodeContent) -> NodeId {
        let id = self.new_id();
        let node = Node::new(id, node_type, content);
        self.nodes.insert(id, node);
        self.append_child(parent_id, id);
        id
    }

    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) -> bool {
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.children.push(child_id);
            if let Some(child) = self.nodes.get_mut(&child_id) {
                child.parent = Some(parent_id);
            }
            return true;
        }
        false
    }

    pub fn delete_node(&mut self, id: NodeId) -> bool {
        if !self.contains(id) || id == self.root { return false; }
        let children = self.nodes.get(&id).map(|n| n.children.clone());
        if let Some(children) = children {
            for child in children { self.delete_node(child); }
        }
        if let Some(parent_id) = self.nodes.get(&id).and_then(|n| n.parent) {
            self.remove_child(parent_id, id);
        }
        self.nodes.remove(&id).is_some()
    }

    pub fn remove_child(&mut self, parent_id: NodeId, child_id: NodeId) -> bool {
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            if let Some(pos) = parent.children.iter().position(|&x| x == child_id) {
                parent.children.remove(pos);
                if let Some(child) = self.nodes.get_mut(&child_id) { child.parent = None; }
                return true;
            }
        }
        false
    }

    pub fn contains(&self, id: NodeId) -> bool { self.nodes.contains_key(&id) }
    pub fn get(&self, id: NodeId) -> Option<&Node> { self.nodes.get(&id) }
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> { self.nodes.get_mut(&id) }
    pub fn root(&self) -> &Node { self.nodes.get(&self.root).unwrap() }
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn is_empty(&self) -> bool { self.nodes.len() <= 1 }
    pub fn children(&self, id: NodeId) -> Option<&Vec<NodeId>> {
        self.nodes.get(&id).map(|n| &n.children)
    }
    pub fn parent(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id).and_then(|n| n.parent).and_then(|pid| self.nodes.get(&pid))
    }
    pub fn next_sibling(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id).and_then(|n| n.parent)
            .and_then(|pid| self.nodes.get(&pid))
            .and_then(|p| {
                let children = &p.children;
                let pos = children.iter().position(|&x| x == id)?;
                children.get(pos + 1).copied()
            })
            .and_then(|sid| self.nodes.get(&sid))
    }
    pub fn depth(&self, id: NodeId) -> usize {
        let mut depth = 0;
        let mut current = id;
        while let Some(parent_id) = self.nodes.get(&current).and_then(|n| n.parent) {
            depth += 1;
            current = parent_id;
        }
        depth
    }
}

impl Default for TreeDoc {
    fn default() -> Self { Self::new() }
}

// ============================================
// Schema
// ============================================

/// Document schema for validation
#[derive(Debug, Default)]
pub struct Schema {
    blocks: Vec<NodeType>,
}

impl Schema {
    pub fn new() -> Self {
        Self {
            blocks: vec![
                NodeType::Document,
                NodeType::Paragraph,
                NodeType::Heading { level: 1 },
                NodeType::List { ordered: false },
                NodeType::Blockquote,
                NodeType::CodeBlock { lang: None },
                NodeType::Divider,
            ],
        }
    }

    pub fn is_valid(&self, node_type: &NodeType) -> bool {
        self.blocks.contains(node_type)
    }
}

// ============================================
// Cursor / Selection
// ============================================

/// A single position in the document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Anchor {
    pub node_id: NodeId,
    pub offset: usize,
}

impl Anchor {
    pub fn new(node_id: NodeId, offset: usize) -> Self { Self { node_id, offset } }
    pub fn at_start(node_id: NodeId) -> Self { Self { node_id, offset: 0 } }
}

/// A selection range
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub anchor: Anchor,
    pub focus: Anchor,
}

impl Selection {
    pub fn caret(anchor: Anchor) -> Self { Self { anchor, focus: anchor } }
    pub fn range(anchor: Anchor, focus: Anchor) -> Self { Self { anchor, focus } }
    pub fn is_caret(&self) -> bool { self.anchor == self.focus }
}

// ============================================
// Operations
// ============================================

/// Document operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    InsertText { node: u64, offset: usize, text: String },
    DeleteText { node: u64, from: usize, to: usize },
    Delete { node: u64 },
    SetSelection { selection: Option<SelectionData> },
}

/// Serializable selection
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionData {
    pub anchor: AnchorData,
    pub focus: AnchorData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorData { pub node_id: u64, pub offset: usize }

/// Operation stack for undo/redo
#[derive(Debug, Clone)]
pub struct OperationStack {
    done: Vec<Operation>,
    undone: Vec<Operation>,
    max_size: usize,
}

impl OperationStack {
    pub fn new(max_size: usize) -> Self { Self { done: Vec::new(), undone: Vec::new(), max_size } }
    pub fn push(&mut self, op: Operation) {
        self.done.push(op.clone());
        self.undone.clear();
        if self.done.len() > self.max_size { self.done.remove(0); }
    }
    pub fn pop(&mut self) -> Option<Operation> { self.done.pop() }
    pub fn can_undo(&self) -> bool { !self.done.is_empty() }
    pub fn can_redo(&self) -> bool { !self.undone.is_empty() }
}

// ============================================
// Main Exports
// ============================================

pub use TreeDoc as Document;
