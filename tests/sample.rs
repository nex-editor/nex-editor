use nex_editor::editor::NexEditor;

pub fn sample1 () -> (NexEditor, &'static str) {
    let mut editor = NexEditor:: new();
    let string = "hello world";

    // append hello world
    if let Some(paragraph_node_key) = editor.append_paragraph_node() {
        for c in string.chars() {
            editor.append_text_node(paragraph_node_key, c);
        }
    }

    (editor, string)
}