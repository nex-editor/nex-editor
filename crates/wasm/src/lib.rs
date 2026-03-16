use runtime::{EditorEvent, EditorRuntime};
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

    pub fn dispatch_json(&mut self, event_json: String) -> String {
        let event: EditorEvent = serde_json::from_str(&event_json).expect("event json should be valid");
        serde_json::to_string(&self.runtime.dispatch(event)).expect("snapshot should serialize")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_editor_smoke() {
        let mut editor = WasmEditor::new();
        let json = editor.dispatch_json(
            serde_json::to_string(&EditorEvent::InsertText { text: "hello".to_string() }).unwrap()
        );
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        assert_eq!(value["text"], "hello");

        let json = editor.dispatch_json(
            serde_json::to_string(&EditorEvent::InsertText { text: "!".to_string() }).unwrap()
        );
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["text"], "hello!");
    }
}
