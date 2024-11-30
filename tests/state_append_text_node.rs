use nex_editor::editor::editor_node::NodeType;
use nex_editor::nodes::root_node::ROOT_NODE_KEY;
use sample::sample1;

pub mod sample;

#[test]
fn state_append_text_node() {
    let (editor, string) = sample1();

    editor.state.print_node_map();

    assert_eq!(editor.state.node_map.len(), 13);

    // check the paragraph node
    let paragraph_node = editor.state.node_map.get(&1).unwrap();
    assert_eq!(paragraph_node.get_first_node_key(), Some(2));
    assert_eq!(paragraph_node.get_last_node_key(), Some(12));
    assert_eq!(paragraph_node.get_parent_node_key(), Some(ROOT_NODE_KEY));

    // check the text nodes
    for i in 2..13 {
        let text_node = editor.state.node_map.get(&i).unwrap();
        assert_eq!(text_node.is_type(NodeType::TextNode), true);
        assert_eq!(text_node.get_parent_node_key(), Some(1));
        assert_eq!(text_node.get_char(), string.chars().nth((i - 2) as usize).expect("char not found"));
        if i == 2 {
            assert_eq!(text_node.get_prev_node_key(), None);
        } else {
            assert_eq!(text_node.get_prev_node_key(), Some(i - 1));
        }
    }
}