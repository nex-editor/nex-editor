use nex_editor::editor::NexEditor;

#[test]
fn test_create_single_paragraph_node() {
    let mut editor = NexEditor:: new();
    editor.append_paragraph_node();
    assert_eq!(editor.state.node_map.len(), 2);
    assert!(editor.state.node_map.contains_key(&0));
    assert!(editor.state.node_map.contains_key(&1));
    let root_node = editor.state.node_map.get(&0).unwrap();
    assert!(root_node.get_first().is_some());
    assert!(root_node.get_last().is_some());
    let first = root_node.get_first().unwrap();
    let last = root_node.get_last().unwrap();
    assert_eq!(first, 1);
    assert_eq!(last, 1);
    editor.state.print_node_map();
}
