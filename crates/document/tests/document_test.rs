use document::{Document, NodeId, NodeType, NodeContent, Anchor, Selection, Schema, Operation, OperationStack};

#[test]
fn test_new_document() {
    let doc = Document::new();
    assert!(doc.is_empty());
    assert_eq!(doc.len(), 1); // Only root
}

#[test]
fn test_create_paragraph() {
    let mut doc = Document::new();
    let root_id = doc.root().id;
    let id = doc.create_node(NodeType::Paragraph, NodeContent::Empty);

    assert!(doc.contains(id));
    assert_eq!(doc.len(), 2);

    let node = doc.get(id).unwrap();
    assert_eq!(node.node_type, NodeType::Paragraph);
}

#[test]
fn test_append_node() {
    let mut doc = Document::new();
    let root_id = doc.root().id;
    let p1 = doc.append_node(root_id, NodeType::Paragraph, NodeContent::Empty);
    let p2 = doc.append_node(root_id, NodeType::Paragraph, NodeContent::Empty);

    assert_eq!(doc.len(), 3);
    assert_ne!(p1, p2);
}

#[test]
fn test_delete_node() {
    let mut doc = Document::new();
    let root_id = doc.root().id;
    let p1 = doc.append_node(root_id, NodeType::Paragraph, NodeContent::Empty);
    let p2 = doc.append_node(root_id, NodeType::Paragraph, NodeContent::Empty);

    assert_eq!(doc.len(), 3);

    let removed = doc.delete_node(p1);
    assert!(removed);
    assert_eq!(doc.len(), 2);
    assert!(!doc.contains(p1));
    assert!(doc.contains(p2));
}

#[test]
fn test_children() {
    let mut doc = Document::new();
    let root_id = doc.root().id;
    let p1 = doc.append_node(root_id, NodeType::Paragraph, NodeContent::Empty);
    let h1 = doc.append_node(root_id, NodeType::Heading { level: 1 }, NodeContent::Empty);

    let children = doc.children(root_id).unwrap();
    assert_eq!(children.len(), 2);
    assert!(children.contains(&p1));
    assert!(children.contains(&h1));
}

#[test]
fn test_sibling_traversal() {
    let mut doc = Document::new();
    let root_id = doc.root().id;
    let p1 = doc.append_node(root_id, NodeType::Paragraph, NodeContent::Empty);
    let p2 = doc.append_node(root_id, NodeType::Paragraph, NodeContent::Empty);
    let p3 = doc.append_node(root_id, NodeType::Paragraph, NodeContent::Empty);

    assert!(doc.next_sibling(p1).is_some());
    assert!(doc.next_sibling(p2).is_some());
    assert!(doc.next_sibling(p3).is_none());
}

#[test]
fn test_depth() {
    let mut doc = Document::new();
    let root_id = doc.root().id;
    let list = doc.append_node(root_id, NodeType::List { ordered: false }, NodeContent::Empty);
    let item = doc.append_node(list, NodeType::ListItem, NodeContent::Empty);
    let p = doc.append_node(item, NodeType::Paragraph, NodeContent::Empty);

    assert_eq!(doc.depth(root_id), 0);
    assert_eq!(doc.depth(list), 1);
    assert_eq!(doc.depth(item), 2);
    assert_eq!(doc.depth(p), 3);
}

#[test]
fn test_parent() {
    let mut doc = Document::new();
    let root_id = doc.root().id;
    let p1 = doc.append_node(root_id, NodeType::Paragraph, NodeContent::Empty);

    let parent = doc.parent(p1).unwrap();
    assert_eq!(parent.id, root_id);
}

#[test]
fn test_node_types() {
    assert!(NodeType::Paragraph.is_block());
    assert!(!NodeType::Paragraph.is_inline());
    assert!(NodeType::Text.is_inline());
    assert!(!NodeType::Text.is_block());
    assert!(NodeType::Text.is_leaf());
    assert!(!NodeType::Paragraph.is_leaf());
}

// === Cursor Tests ===

#[test]
fn test_anchor_creation() {
    let anchor = Anchor::new(NodeId(1), 5);
    assert_eq!(anchor.node_id, NodeId(1));
    assert_eq!(anchor.offset, 5);
}

#[test]
fn test_anchor_at_start() {
    let anchor = Anchor::at_start(NodeId(1));
    assert_eq!(anchor.node_id, NodeId(1));
    assert_eq!(anchor.offset, 0);
}

#[test]
fn test_selection_caret() {
    let anchor = Anchor::new(NodeId(1), 5);
    let selection = Selection::caret(anchor);
    assert!(selection.is_caret());
}

#[test]
fn test_selection_range() {
    let anchor1 = Anchor::new(NodeId(1), 0);
    let anchor2 = Anchor::new(NodeId(1), 5);
    let selection = Selection::range(anchor1, anchor2);
    assert!(!selection.is_caret());
}

// === Schema Tests ===

#[test]
fn test_schema() {
    let schema = Schema::new();
    assert!(schema.is_valid(&NodeType::Paragraph));
    assert!(schema.is_valid(&NodeType::Heading { level: 1 }));
    assert!(schema.is_valid(&NodeType::List { ordered: false }));
}

#[test]
fn test_schema_default() {
    let schema = Schema::new();
    // Schema::new() should include Heading level 1
    assert!(schema.is_valid(&NodeType::Heading { level: 1 }));
}

// === Operation Stack Tests ===

#[test]
fn test_operation_stack() {
    let stack = OperationStack::new(100);
    assert!(!stack.can_undo());
    assert!(!stack.can_redo());
}

#[test]
fn test_operation_stack_push_pop() {
    let mut stack = OperationStack::new(10);
    let op = Operation::InsertText { node: 1, offset: 0, text: "hello".to_string() };

    stack.push(op.clone());
    assert!(stack.can_undo());
    assert!(!stack.can_redo());

    let popped = stack.pop().unwrap();
    assert_eq!(popped, op);
}

#[test]
fn test_operation_stack_max_size() {
    let mut stack = OperationStack::new(3);

    for i in 0..5 {
        let op = Operation::InsertText { node: i, offset: 0, text: format!("{}", i) };
        stack.push(op);
    }

    assert!(!stack.can_redo());
}
