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

    pub fn debug_snapshot_json(&self) -> String {
        serde_json::to_string(&self.runtime.debug_snapshot()).expect("debug snapshot should serialize")
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

    #[test]
    fn test_wasm_editor_exposes_debug_snapshot() {
        let editor = WasmEditor::new();
        let json = editor.debug_snapshot_json();
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        assert_eq!(value["revision"], 0);
        assert_eq!(value["doc"]["block_type"], "doc");
    }

    #[test]
    fn test_wasm_editor_snapshot_exposes_style_table() {
        let editor = WasmEditor::new();
        let json = editor.snapshot_json();
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        assert_eq!(value["scene"]["styles"][0]["id"], "surface");
        assert_eq!(value["scene"]["background"][0]["style_id"], "surface");
    }

    #[test]
    fn test_wasm_editor_debug_snapshot_exposes_layout_metrics() {
        let editor = WasmEditor::new();
        let json = editor.debug_snapshot_json();
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        assert_eq!(value["layout"]["font_size_px"], 18.0);
        assert_eq!(value["layout"]["caret_width"], 2.0);
        assert_eq!(value["layout"]["ascent"], 18.0);
        assert_eq!(value["layout"]["descent"], 4.0);
    }
}
