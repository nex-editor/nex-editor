//! Tree-based document structure using Loro CRDT.
//!
//! This module provides a tree-based document model where:
//! - Each node has a unique ID
//! - Nodes form a tree hierarchy (parent/child relationships)
//! - Loro handles collaborative editing and conflict resolution
//!
//! # Architecture
//!
//! ```
//! Document
//! ├── Node (root)
//! │   ├── Node (paragraph)
//! │   │   └── Node (text)
//! │   ├── Node (heading)
//! │   └── Node (list)
//! │       ├── Node (list_item)
//! │       └── Node (list_item)
//! ```

use crate::tree::node::*;
use crate::LoroDoc;
use loro::TreeHandler;
use loro_internal::ID;
use std::collections::HashMap;

/// ID type used by Loro internally for tree nodes
pub type TreeID = ID;

/// The tree-based document
#[derive(Debug)]
pub struct TreeDoc {
    /// The underlying Loro document
    doc: LoroDoc,
    /// Root node ID
    root: NodeId,
    /// All nodes in the document
    nodes: HashMap<NodeId, Node>,
    /// Counter for generating new node IDs
    id_counter: u64,
}

impl TreeDoc {
    /// Create a new empty document
    pub fn new() -> Self {
        let doc = LoroDoc::new();
        let root = NodeId(0); // Root node at ID 0

        let mut nodes = HashMap::new();
        nodes.insert(
            root,
            Node::new(root, NodeType::Document, NodeContent::Empty),
        );

        Self {
            doc,
            root,
            nodes,
            id_counter: 1,
        }
    }

    /// Get the underlying LoroDoc for sync/export/import
    pub fn loro_doc(&self) -> &LoroDoc {
        &self.doc
    }

    /// Generate a new unique node ID
    fn new_id(&mut self) -> NodeId {
        let id = self.id_counter;
        self.id_counter += 1;
        NodeId(id)
    }

    // === Node Operations ===

    /// Create a new node
    pub fn create_node(
        &mut self,
        node_type: NodeType,
        content: NodeContent,
    ) -> NodeId {
        let id = self.new_id();
        let node = Node::new(id, node_type, content);
        self.nodes.insert(id, node);
        id
    }

    /// Create a node and attach it to a parent
    pub fn append_node(
        &mut self,
        parent_id: NodeId,
        node_type: NodeType,
        content: NodeContent,
    ) -> NodeId {
        let id = self.create_node(node_type, content);
        self.append_child(parent_id, id);
        id
    }

    /// Insert a node at a specific position in parent's children
    pub fn insert_node(
        &mut self,
        parent_id: NodeId,
        index: usize,
        node_type: NodeType,
        content: NodeContent,
    ) -> Option<NodeId> {
        if !self.contains(parent_id) {
            return None;
        }

        let id = self.create_node(node_type, content);

        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            if index >= parent.children.len() {
                parent.children.push(id);
            } else {
                parent.children.insert(index, id);
            }
        }

        if let Some(node) = self.nodes.get_mut(&id) {
            node.parent = Some(parent_id);
        }

        Some(id)
    }

    /// Add a child to a parent
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) -> bool {
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            if parent.can_have_children() {
                parent.children.push(child_id);

                if let Some(child) = self.nodes.get_mut(&child_id) {
                    child.parent = Some(parent_id);
                }
                return true;
            }
        }
        false
    }

    /// Remove a node from its parent
    pub fn remove_child(&mut self, parent_id: NodeId, child_id: NodeId) -> bool {
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            let removed = parent.children.iter().position(|&x| x == child_id);
            if let Some(pos) = removed {
                parent.children.remove(pos);
                if let Some(child) = self.nodes.get_mut(&child_id) {
                    child.parent = None;
                }
                return true;
            }
        }
        false
    }

    /// Delete a node (removes from parent and all children)
    pub fn delete_node(&mut self, id: NodeId) -> bool {
        if !self.contains(id) || id == self.root {
            return false;
        }

        // Recursively delete children
        let children = {
            let node = self.nodes.get(&id)?;
            node.children.clone()
        };

        for child in children {
            self.delete_node(child);
        }

        // Remove from parent
        if let Some(parent_id) = self.nodes.get(&id).and_then(|n| n.parent) {
            self.remove_child(parent_id, id);
        }

        // Mark as deleted
        if let Some(node) = self.nodes.get_mut(&id) {
            node.meta.deleted = true;
        }

        self.nodes.remove(&id).is_some()
    }

    // === Query Operations ===

    /// Check if a node exists
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    /// Get a node by ID (immutable)
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get a node by ID (mutable)
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// Get the root node
    pub fn root(&self) -> &Node {
        self.nodes.get(&self.root).unwrap()
    }

    /// Get node's parent
    pub fn parent(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
            .and_then(|n| n.parent)
            .and_then(|pid| self.nodes.get(&pid))
    }

    /// Get node's children
    pub fn children(&self, id: NodeId) -> Option<&Vec<NodeId>> {
        self.nodes.get(&id).map(|n| &n.children)
    }

    /// Get the next sibling
    pub fn next_sibling(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
            .and_then(|n| n.parent)
            .and_then(|pid| self.nodes.get(&pid))
            .and_then(|p| {
                let children = &p.children;
                let pos = children.iter().position(|&x| x == id)?;
                children.get(pos + 1).copied()
            })
            .and_then(|sid| self.nodes.get(&sid))
    }

    /// Get the previous sibling
    pub fn prev_sibling(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
            .and_then(|n| n.parent)
            .and_then(|pid| self.nodes.get(&pid))
            .and_then(|p| {
                let children = &p.children;
                let pos = children.iter().position(|&x| x == id)?;
                if pos == 0 { None } else { children.get(pos - 1).copied() }
            })
            .and_then(|sid| self.nodes.get(&sid))
    }

    /// Count nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.nodes.len() <= 1 // Only root
    }

    // === Iteration ===

    /// Get an iterator over all nodes
    pub fn iter(&self) -> impl Iterator<Item = &Node> {
        self.nodes.values()
    }

    /// Get a mutable iterator
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&NodeId, &mut Node)> {
        self.nodes.iter_mut()
    }

    /// Get children as nodes
    pub fn children_nodes(&self, id: NodeId) -> Option<Vec<&Node>> {
        self.nodes.get(&id).map(|n| {
            n.children.iter()
                .filter_map(|cid| self.nodes.get(cid))
                .collect()
        })
    }

    // === Utility ===

    /// Get the depth of a node (root = 0)
    pub fn depth(&self, id: NodeId) -> usize {
        let mut depth = 0;
        let mut current = id;
        while let Some(parent_id) = self.nodes.get(&current).and_then(|n| n.parent) {
            depth += 1;
            current = parent_id;
        }
        depth
    }

    /// Check if a node is an ancestor of another
    pub fn is_ancestor(&self, ancestor_id: NodeId, descendant_id: NodeId) -> bool {
        let mut current = descendant_id;
        while let Some(parent_id) = self.nodes.get(&current).and_then(|n| n.parent) {
            if parent_id == ancestor_id {
                return true;
            }
            current = parent_id;
        }
        false
    }

    /// Get all blocks (depth-first order)
    pub fn blocks(&self) -> Vec<&Node> {
        self.collect_blocks(self.root)
    }

    fn collect_blocks(&self, id: NodeId) -> Vec<&Node> {
        let mut blocks = Vec::new();
        if let Some(node) = self.nodes.get(&id) {
            if node.node_type.is_block() && !node.meta.deleted {
                blocks.push(node);
            }
            for &child in &node.children {
                blocks.extend(self.collect_blocks(child));
            }
        }
        blocks
    }
}

impl Default for TreeDoc {
    fn default() -> Self {
        Self::new()
    }
}

// Re-exports
pub use crate::tree::node::*;
pub use crate::tree::iterator::*;
pub use crate::tree::builder::*;
