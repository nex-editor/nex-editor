use runtime::{EditorRuntime, InputCommand};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmEditor {
    runtime: EditorRuntime,
}

#[wasm_bindgen]
impl WasmEditor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            runtime: EditorRuntime::new(),
        }
    }

    pub fn snapshot_json(&self) -> String {
        serde_json::to_string(&self.runtime.snapshot()).expect("snapshot should serialize")
    }

    pub fn set_text(&mut self, text: String) -> String {
        self.apply(InputCommand::SetText { text })
    }

    pub fn set_selection(&mut self, anchor: usize, head: usize) -> String {
        self.apply(InputCommand::SetSelection { anchor, head })
    }

    pub fn insert_text(&mut self, text: String) -> String {
        self.apply(InputCommand::InsertText { text })
    }

    pub fn backspace(&mut self) -> String {
        self.apply(InputCommand::Backspace)
    }

    pub fn delete_forward(&mut self) -> String {
        self.apply(InputCommand::DeleteForward)
    }

    pub fn select_all(&mut self) -> String {
        self.apply(InputCommand::SelectAll)
    }
}

impl WasmEditor {
    fn apply(&mut self, command: InputCommand) -> String {
        serde_json::to_string(&self.runtime.apply(command)).expect("snapshot should serialize")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_editor_smoke() {
        let mut editor = WasmEditor::new();
        let json = editor.set_text("hello".to_string());
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        assert_eq!(value["text"], "hello");

        let json = editor.insert_text("!".to_string());
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["text"], "hello!");
    }
}
