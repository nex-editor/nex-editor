pub mod sample;

use nex_editor::editor::editor_node::NodeType;
use nex_editor::editor::NexEditor;
use nex_editor::nodes::root_node::ROOT_NODE_KEY;
use sample::sample1;

#[test]
fn test_append_paragraph_node() {
    // append a paragraph node
    let mut editor = NexEditor::new();
    let paragraph_node_key = editor.append_paragraph_node();
    assert_eq!(paragraph_node_key.is_some(), true);

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

    // append another paragraph node
    let paragraph_node_key = editor.append_paragraph_node();
    assert_eq!(paragraph_node_key.is_some(), true);

    editor.state.print_node_map();
}

#[test]
fn test_append_text_node() {
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