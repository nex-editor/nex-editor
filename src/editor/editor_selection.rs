#[derive(Debug)]
pub struct EditorSelection {
    start: u32,
    end: u32,
    cursor: u32,
}

pub fn create_empty_editor_selection() -> EditorSelection {
    EditorSelection {
        start: 0,
        end: 0,
        cursor: 0,
    }
}
