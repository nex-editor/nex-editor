use nex_editor::editor::NexEditor;

#[test]
fn test_create_single_paragraph_node() {
    // append a paragraph node
    let mut editor = NexEditor:: new();
    editor.append_paragraph_node();

    assert_eq!(editor.state.node_map.len(), 2);
    assert!(editor.state.node_map.contains_key(&0));
    assert!(editor.state.node_map.contains_key(&1));

    let root_node = editor.state.node_map.get(&0).unwrap();
    assert!(root_node.get_first_key().is_some());
    assert!(root_node.get_last_key().is_some());

    // append again
    editor.append_paragraph_node();

    editor.state.print_node_map();
}
