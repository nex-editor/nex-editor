use nex_editor::editor::NexEditor;

#[test]
fn test_create_single_paragraph_node() {
    let mut editor = NexEditor:: new();
    editor.append_paragraph_node();
    // 一定有两个节点
    assert!(editor.state.node_map.len() == 2);
    // 一定有一个根节点
    assert!(editor.state.node_map.contains_key(&0));
    // 一个段落节点
    assert!(editor.state.node_map.contains_key(&1));
    // 根节点的first和last指向段落节点
    let root_node = editor.state.node_map.get(&0).unwrap();
    assert!(root_node.get_first().is_some());
    assert!(root_node.get_last().is_some());
    let first = root_node.get_first().unwrap();
    let last = root_node.get_last().unwrap();
    assert!(first == 1);
    assert!(last == 1);
    editor.state.print_node_map();
}
