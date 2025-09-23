use document::Document;

pub struct EditorState {
    pub doc: Document,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            doc: Document::new(),
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}