#[derive(Debug)]
pub struct EditorSelection {
    // 起始位置
    start: u32,
    // 结束位置
    end: u32,
    // 当前位置
    cursor: u32,
}

pub fn create_empty_editor_selection() -> EditorSelection {
    EditorSelection {
        start: 0,
        end: 0,
        cursor: 0,
    }
}
