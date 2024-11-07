use nex_editor::editor::editor_node::NodeType;
use nex_editor::editor::NexEditor;
use nex_editor::nodes::root_node::ROOT_NODE_KEY;

#[test]
fn test_create_single_paragraph_node() {
    // append a paragraph node
    let mut editor = NexEditor:: new();
    editor.append_paragraph_node();

    // check the node map
    // length must be 2
    assert_eq!(editor.state.node_map.len(), 2);

    // root node
    let root_node = editor.state.node_map.get(&ROOT_NODE_KEY).unwrap();
    assert_eq!(root_node.is_type(NodeType::RootNode), true);
    assert_eq!(root_node.get_first_node_key(), Some(1));
    assert_eq!(root_node.get_last_node_key(), Some(1));

    // paragraph node
    let paragraph_node = editor.state.node_map.get(&1).unwrap();
    assert_eq!(paragraph_node.is_type(NodeType::ParagraphNode), true);
    assert_eq!(paragraph_node.get_first_node_key(), None);
    assert_eq!(paragraph_node.get_last_node_key(), None);
    assert_eq!(paragraph_node.get_parent_node_key(), Some(ROOT_NODE_KEY));

    editor.state.print_node_map();
}
