//! Headless editor runtime with layout and hit-testing owned by Rust.

mod commands;
mod editing;
mod layout;
mod navigation;
mod pointer;

use model::Node;
use serde::{Deserialize, Serialize};
use state::EditorState;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub device_pixel_ratio: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub padding_x: f32,
    pub padding_y: f32,
    pub line_height: f32,
    pub char_width: f32,
    pub caret_width: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            padding_x: 24.0,
            padding_y: 28.0,
            line_height: 28.0,
            char_width: 9.6,
            caret_width: 2.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextRun {
    pub text: String,
    pub start: usize,
    pub end: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineLayout {
    pub line_index: usize,
    pub start: usize,
    pub end: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub runs: Vec<TextRun>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SelectionRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CaretLayout {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderSnapshot {
    pub text: String,
    pub selection_anchor: usize,
    pub selection_head: usize,
    pub revision: u64,
    pub viewport: Viewport,
    pub content_width: f32,
    pub content_height: f32,
    pub lines: Vec<LineLayout>,
    pub selection_rects: Vec<SelectionRect>,
    pub caret: Option<CaretLayout>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextLayout {
    pub lines: Vec<LineLayout>,
    pub content_width: f32,
    pub content_height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlatTextOffset(pub usize);

impl FlatTextOffset {
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    pub fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HitTestResult {
    pub offset: FlatTextOffset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Modifiers {
    pub shift: bool,
    pub alt: bool,
    pub meta: bool,
    pub ctrl: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EditorEvent {
    ResizeViewport { width: f32, height: f32, device_pixel_ratio: f32 },
    PointerDown { x: f32, y: f32, button: PointerButton, modifiers: Modifiers, click_count: u8 },
    PointerMove { x: f32, y: f32, modifiers: Modifiers },
    PointerUp { x: f32, y: f32, button: PointerButton, modifiers: Modifiers },
    InsertText { text: String },
    Backspace,
    DeleteForward,
    MoveCaretLeft,
    MoveCaretRight,
    MoveCaretUp,
    MoveCaretDown,
    SelectAll,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditorRuntime {
    state: EditorState,
    revision: u64,
    viewport: Viewport,
    layout: LayoutConfig,
    pointer_anchor: Option<FlatTextOffset>,
}

impl Default for EditorRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorRuntime {
    pub fn new() -> Self {
        Self {
            state: EditorState::from_doc(Node::from_paragraph_texts(vec![String::new()])),
            revision: 0,
            viewport: Viewport {
                width: 900.0,
                height: 480.0,
                device_pixel_ratio: 1.0,
            },
            layout: LayoutConfig::default(),
            pointer_anchor: None,
        }
    }

    pub fn snapshot(&self) -> RenderSnapshot {
        let text = self.state.doc().plain_text();
        let text_layout = self.text_layout(&text);
        let selection_anchor = self.state.selection().anchor();
        let selection_head = self.state.selection().head();

        RenderSnapshot {
            text,
            selection_anchor,
            selection_head,
            revision: self.revision,
            viewport: self.viewport,
            content_width: text_layout.content_width,
            content_height: text_layout.content_height,
            selection_rects: self.selection_rects(&text_layout, selection_anchor, selection_head),
            caret: self.caret_layout(&text_layout, selection_anchor, selection_head),
            lines: text_layout.lines,
        }
    }

    pub fn dispatch(&mut self, event: EditorEvent) -> RenderSnapshot {
        match event {
            EditorEvent::ResizeViewport { width, height, device_pixel_ratio } => {
                self.viewport = Viewport { width, height, device_pixel_ratio };
            }
            EditorEvent::PointerDown { x, y, button, .. } => self.pointer_down(x, y, button),
            EditorEvent::PointerMove { x, y, .. } => self.pointer_move(x, y),
            EditorEvent::PointerUp { .. } => self.pointer_up(),
            EditorEvent::InsertText { text } => self.insert_text(text),
            EditorEvent::Backspace => self.backspace(),
            EditorEvent::DeleteForward => self.delete_forward(),
            EditorEvent::MoveCaretLeft => self.move_caret_left(),
            EditorEvent::MoveCaretRight => self.move_caret_right(),
            EditorEvent::MoveCaretUp => self.move_caret_vertical(-1),
            EditorEvent::MoveCaretDown => self.move_caret_vertical(1),
            EditorEvent::SelectAll => self.select_all(),
        }

        self.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_text_updates_render_snapshot() {
        let mut runtime = EditorRuntime::new();
        let snapshot = runtime.dispatch(EditorEvent::InsertText { text: "hello".to_string() });

        assert_eq!(snapshot.text, "hello");
        assert_eq!(snapshot.selection_anchor, 5);
        assert_eq!(snapshot.selection_head, 5);
        assert_eq!(snapshot.revision, 1);
        assert_eq!(snapshot.lines.len(), 1);
    }

    #[test]
    fn test_pointer_selection_uses_runtime_hit_testing() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::InsertText { text: "hello".to_string() });

        let snapshot = runtime.dispatch(EditorEvent::PointerDown {
            x: 24.0 + 2.0 * 9.6,
            y: 28.0,
            button: PointerButton::Primary,
            modifiers: Modifiers::default(),
            click_count: 1,
        });

        assert_eq!(snapshot.selection_anchor, 2);
        assert_eq!(snapshot.selection_head, 2);
    }

    #[test]
    fn test_drag_selection_produces_selection_rects() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::InsertText { text: "hello".to_string() });
        runtime.dispatch(EditorEvent::PointerDown {
            x: 24.0,
            y: 28.0,
            button: PointerButton::Primary,
            modifiers: Modifiers::default(),
            click_count: 1,
        });
        let snapshot = runtime.dispatch(EditorEvent::PointerMove {
            x: 24.0 + 3.0 * 9.6,
            y: 28.0,
            modifiers: Modifiers::default(),
        });

        assert_eq!(snapshot.selection_anchor, 0);
        assert_eq!(snapshot.selection_head, 3);
        assert_eq!(snapshot.selection_rects.len(), 1);
    }

    #[test]
    fn test_resize_reflows_lines() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::InsertText { text: "abcdef".to_string() });
        let snapshot = runtime.dispatch(EditorEvent::ResizeViewport {
            width: 24.0 * 2.0 + 2.0 * 9.6,
            height: 480.0,
            device_pixel_ratio: 1.0,
        });

        assert!(snapshot.lines.len() >= 3);
    }

    #[test]
    fn test_select_all_does_not_increment_revision() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::InsertText { text: "abc".to_string() });
        let snapshot = runtime.dispatch(EditorEvent::SelectAll);

        assert_eq!(snapshot.selection_anchor, 0);
        assert_eq!(snapshot.selection_head, 3);
        assert_eq!(snapshot.revision, 1);
    }

    #[test]
    fn test_vertical_movement_uses_visual_lines() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::ResizeViewport {
            width: 24.0 * 2.0 + 3.0 * 9.6,
            height: 480.0,
            device_pixel_ratio: 1.0,
        });
        runtime.dispatch(EditorEvent::InsertText { text: "abcdef".to_string() });
        runtime.dispatch(EditorEvent::PointerDown {
            x: 24.0 + 2.0 * 9.6,
            y: 28.0,
            button: PointerButton::Primary,
            modifiers: Modifiers::default(),
            click_count: 1,
        });

        let down = runtime.dispatch(EditorEvent::MoveCaretDown);
        assert_eq!(down.selection_anchor, 5);
        assert_eq!(down.selection_head, 5);

        let up = runtime.dispatch(EditorEvent::MoveCaretUp);
        assert_eq!(up.selection_anchor, 2);
        assert_eq!(up.selection_head, 2);
    }

    #[test]
    fn test_snapshot_includes_content_metrics() {
        let mut runtime = EditorRuntime::new();
        let snapshot = runtime.dispatch(EditorEvent::InsertText { text: "abc\ndef".to_string() });

        assert!(snapshot.content_width >= 48.0);
        assert!(snapshot.content_height >= 112.0);
        assert_eq!(snapshot.lines.len(), 2);
    }

    #[test]
    fn test_inserting_newline_creates_multiple_paragraphs() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::InsertText { text: "abc\ndef".to_string() });

        assert_eq!(runtime.state.doc().paragraph_texts(), vec!["abc".to_string(), "def".to_string()]);
        assert_eq!(runtime.state.doc().plain_text(), "abc\ndef");
    }
}
