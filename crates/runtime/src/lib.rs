//! Headless editor runtime with layout and hit-testing owned by Rust.

mod commands;
mod editing;
mod layout;
mod navigation;
mod pointer;

use model::Node;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use state::EditorState;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
    pub device_pixel_ratio: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub padding_x: f32,
    pub padding_y: f32,
    pub line_height: f32,
    pub char_width: f32,
    pub caret_width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub font_size_px: f32,
    pub font_family: String,
    pub text_style_key: String,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            padding_x: 24.0,
            padding_y: 28.0,
            line_height: 28.0,
            char_width: 9.6,
            caret_width: 2.0,
            ascent: 18.0,
            descent: 4.0,
            font_size_px: 18.0,
            font_family: "\"IBM Plex Mono\", monospace".to_string(),
            text_style_key: "text.primary".to_string(),
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
    pub baseline_y: f32,
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
    pub baseline_y: f32,
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
    pub display_text: String,
    pub selection_anchor: usize,
    pub selection_head: usize,
    pub revision: u64,
    pub composition: Option<CompositionSnapshot>,
    pub viewport: Viewport,
    pub content_width: f32,
    pub content_height: f32,
    pub scene: SceneSnapshot,
    pub lines: Vec<LineLayout>,
    pub selection_rects: Vec<SelectionRect>,
    pub caret: Option<CaretLayout>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneSnapshot {
    pub viewport: Viewport,
    pub content_width: f32,
    pub content_height: f32,
    pub styles: Vec<PaintStyle>,
    pub background: Vec<PaintRect>,
    pub selection_rects: Vec<PaintRect>,
    pub composition_underlines: Vec<PaintRect>,
    pub text_runs: Vec<PaintTextRun>,
    pub caret: Option<PaintRect>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaintStyle {
    pub id: String,
    pub role: PaintStyleRole,
    pub measurement_style_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaintRect {
    pub kind: PaintRectKind,
    pub style_id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaintRectKind {
    Background,
    Selection,
    Caret,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaintStyleRole {
    EditorSurface,
    PrimaryText,
    SelectionFill,
    CaretFill,
    CompositionUnderline,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaintTextRun {
    pub text: String,
    pub style_id: String,
    pub x: f32,
    pub baseline_y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugSnapshot {
    pub revision: u64,
    pub text: String,
    pub selection_anchor: usize,
    pub selection_head: usize,
    pub layout: DebugLayoutConfig,
    pub composition: Option<CompositionSnapshot>,
    pub doc: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionState {
    pub from: usize,
    pub to: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompositionSnapshot {
    pub from: usize,
    pub to: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebugLayoutConfig {
    pub font_family: String,
    pub font_size_px: f32,
    pub char_width: f32,
    pub line_height: f32,
    pub caret_width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub text_style_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextMeasurementEntry {
    pub style_key: String,
    pub text: String,
    pub advance: f32,
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
    ResizeViewport {
        width: f32,
        height: f32,
        device_pixel_ratio: f32,
    },
    SetTextMetrics {
        font_family: String,
        font_size_px: f32,
        char_width: f32,
        line_height: f32,
        caret_width: f32,
        ascent: f32,
        descent: f32,
    },
    SetTextMeasurements {
        entries: Vec<TextMeasurementEntry>,
    },
    CompositionStart,
    CompositionUpdate {
        text: String,
    },
    CompositionEnd {
        text: String,
    },
    CompositionCancel,
    PointerDown {
        x: f32,
        y: f32,
        button: PointerButton,
        modifiers: Modifiers,
        click_count: u8,
    },
    PointerMove {
        x: f32,
        y: f32,
        modifiers: Modifiers,
    },
    PointerUp {
        x: f32,
        y: f32,
        button: PointerButton,
        modifiers: Modifiers,
    },
    InsertText {
        text: String,
    },
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
    measurements: HashMap<String, HashMap<String, f32>>,
    pointer_anchor: Option<FlatTextOffset>,
    composition: Option<CompositionState>,
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
            measurements: HashMap::new(),
            pointer_anchor: None,
            composition: None,
        }
    }

    pub fn snapshot(&self) -> RenderSnapshot {
        let text = self.state.doc().plain_text();
        let display_text = self.display_text(&text);
        let composition = self.composition_snapshot();
        let text_layout = self.text_layout(&display_text);
        let (selection_anchor, selection_head) = self.display_selection();
        let selection_rects = self.selection_rects(&text_layout, selection_anchor, selection_head);
        let caret = self.caret_layout(&text_layout, selection_anchor, selection_head);
        let composition_underlines =
            self.composition_underlines(&text_layout, self.composition_range());
        let scene = self.scene_snapshot(
            &text_layout,
            &selection_rects,
            &composition_underlines,
            caret,
        );

        RenderSnapshot {
            text,
            display_text,
            selection_anchor,
            selection_head,
            revision: self.revision,
            composition,
            viewport: self.viewport,
            content_width: text_layout.content_width,
            content_height: text_layout.content_height,
            scene,
            selection_rects,
            caret,
            lines: text_layout.lines,
        }
    }

    pub fn debug_snapshot(&self) -> DebugSnapshot {
        DebugSnapshot {
            revision: self.revision,
            text: self.state.doc().plain_text(),
            selection_anchor: self.state.selection().anchor(),
            selection_head: self.state.selection().head(),
            layout: DebugLayoutConfig {
                font_family: self.layout.font_family.clone(),
                font_size_px: self.layout.font_size_px,
                char_width: self.layout.char_width,
                line_height: self.layout.line_height,
                caret_width: self.layout.caret_width,
                ascent: self.layout.ascent,
                descent: self.layout.descent,
                text_style_key: self.layout.text_style_key.clone(),
            },
            composition: self.composition_snapshot(),
            doc: serde_json::to_value(self.state.doc()).expect("doc should serialize"),
        }
    }

    pub fn dispatch(&mut self, event: EditorEvent) -> RenderSnapshot {
        match event {
            EditorEvent::ResizeViewport {
                width,
                height,
                device_pixel_ratio,
            } => {
                self.viewport = Viewport {
                    width,
                    height,
                    device_pixel_ratio,
                };
            }
            EditorEvent::SetTextMetrics {
                font_family,
                font_size_px,
                char_width,
                line_height,
                caret_width,
                ascent,
                descent,
            } => {
                self.layout.font_family = font_family;
                self.layout.font_size_px = font_size_px;
                self.layout.char_width = char_width.max(1.0);
                self.layout.line_height = line_height.max(font_size_px.max(1.0));
                self.layout.caret_width = caret_width.max(1.0);
                self.layout.ascent = ascent.max(1.0);
                self.layout.descent = descent.max(0.0);
            }
            EditorEvent::SetTextMeasurements { entries } => self.set_text_measurements(entries),
            EditorEvent::CompositionStart => self.start_composition(),
            EditorEvent::CompositionUpdate { text } => self.update_composition(text),
            EditorEvent::CompositionEnd { text } => self.commit_composition(text),
            EditorEvent::CompositionCancel => self.cancel_composition(),
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

    fn composition_snapshot(&self) -> Option<CompositionSnapshot> {
        self.composition
            .as_ref()
            .map(|composition| CompositionSnapshot {
                from: composition.from,
                to: composition.to,
                text: composition.text.clone(),
            })
    }

    fn display_text(&self, committed_text: &str) -> String {
        let Some(composition) = &self.composition else {
            return committed_text.to_string();
        };

        let from_byte = Node::char_to_byte_index(committed_text, composition.from);
        let to_byte = Node::char_to_byte_index(committed_text, composition.to);
        let mut next = String::with_capacity(
            committed_text.len() - (to_byte - from_byte) + composition.text.len(),
        );
        next.push_str(&committed_text[..from_byte]);
        next.push_str(&composition.text);
        next.push_str(&committed_text[to_byte..]);
        next
    }

    fn display_selection(&self) -> (usize, usize) {
        if let Some(composition) = &self.composition {
            let head = composition.from + Node::char_len(&composition.text);
            return (head, head);
        }

        (
            self.state.selection().anchor(),
            self.state.selection().head(),
        )
    }

    fn composition_range(&self) -> Option<(usize, usize)> {
        self.composition.as_ref().map(|composition| {
            let from = composition.from;
            let to = composition.from + Node::char_len(&composition.text);
            (from, to)
        })
    }

    fn set_text_measurements(&mut self, entries: Vec<TextMeasurementEntry>) {
        for entry in entries {
            self.measurements
                .entry(entry.style_key)
                .or_default()
                .insert(entry.text, entry.advance.max(0.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_text_updates_render_snapshot() {
        let mut runtime = EditorRuntime::new();
        let snapshot = runtime.dispatch(EditorEvent::InsertText {
            text: "hello".to_string(),
        });

        assert_eq!(snapshot.text, "hello");
        assert_eq!(snapshot.selection_anchor, 5);
        assert_eq!(snapshot.selection_head, 5);
        assert_eq!(snapshot.revision, 1);
        assert_eq!(snapshot.lines.len(), 1);
    }

    #[test]
    fn test_pointer_selection_uses_runtime_hit_testing() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::InsertText {
            text: "hello".to_string(),
        });

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
        runtime.dispatch(EditorEvent::InsertText {
            text: "hello".to_string(),
        });
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
        runtime.dispatch(EditorEvent::InsertText {
            text: "abcdef".to_string(),
        });
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
        runtime.dispatch(EditorEvent::InsertText {
            text: "abc".to_string(),
        });
        let snapshot = runtime.dispatch(EditorEvent::SelectAll);

        assert_eq!(snapshot.selection_anchor, 0);
        assert_eq!(snapshot.selection_head, 3);
        assert_eq!(snapshot.revision, 1);
    }

    #[test]
    fn test_snapshot_exposes_scene_for_shell_rendering() {
        let mut runtime = EditorRuntime::new();
        let snapshot = runtime.dispatch(EditorEvent::InsertText {
            text: "abc".to_string(),
        });

        assert_eq!(snapshot.scene.text_runs.len(), 3);
        assert_eq!(
            snapshot
                .scene
                .text_runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>(),
            "abc"
        );
        assert_eq!(
            snapshot.scene.text_runs[0].baseline_y,
            snapshot.lines[0].baseline_y
        );
        assert_eq!(snapshot.scene.text_runs[0].style_id, "text.primary");
        assert_eq!(snapshot.scene.background[0].style_id, "surface");
        assert_eq!(snapshot.scene.styles.len(), 5);
        assert_eq!(snapshot.scene.styles[0].role, PaintStyleRole::EditorSurface);
        assert_eq!(snapshot.scene.styles[0].measurement_style_key, None);
        assert_eq!(
            snapshot.scene.styles[1].measurement_style_key.as_deref(),
            Some("text.primary")
        );
        assert_eq!(snapshot.scene.selection_rects.len(), 0);
        assert!(snapshot.scene.caret.is_some());
    }

    #[test]
    fn test_set_text_metrics_updates_layout_inputs() {
        let mut runtime = EditorRuntime::new();
        let _ = runtime.dispatch(EditorEvent::SetTextMetrics {
            font_family: "Test Mono".to_string(),
            font_size_px: 16.0,
            char_width: 8.0,
            line_height: 24.0,
            caret_width: 3.0,
            ascent: 13.0,
            descent: 5.0,
        });
        let debug = runtime.debug_snapshot();

        assert_eq!(debug.layout.font_family, "Test Mono");
        assert_eq!(debug.layout.font_size_px, 16.0);
        assert_eq!(debug.layout.char_width, 8.0);
        assert_eq!(debug.layout.line_height, 24.0);
        assert_eq!(debug.layout.caret_width, 3.0);
        assert_eq!(debug.layout.ascent, 13.0);
        assert_eq!(debug.layout.descent, 5.0);
    }

    #[test]
    fn test_vertical_geometry_uses_baseline_metrics() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::SetTextMetrics {
            font_family: "Test Mono".to_string(),
            font_size_px: 16.0,
            char_width: 8.0,
            line_height: 24.0,
            caret_width: 2.0,
            ascent: 13.0,
            descent: 5.0,
        });
        let snapshot = runtime.dispatch(EditorEvent::InsertText {
            text: "abc".to_string(),
        });

        assert_eq!(snapshot.lines[0].y, 28.0);
        assert_eq!(snapshot.lines[0].baseline_y, 41.0);
        assert_eq!(snapshot.scene.text_runs[0].baseline_y, 41.0);

        let caret = snapshot.scene.caret.expect("caret should exist");
        assert_eq!(caret.y, 28.0);
        assert_eq!(caret.height, 18.0);
    }

    #[test]
    fn test_unicode_input_uses_character_offsets() {
        let mut runtime = EditorRuntime::new();
        let snapshot = runtime.dispatch(EditorEvent::InsertText {
            text: "你a".to_string(),
        });

        assert_eq!(snapshot.text, "你a");
        assert_eq!(snapshot.selection_anchor, 2);
        assert_eq!(snapshot.selection_head, 2);

        let snapshot = runtime.dispatch(EditorEvent::MoveCaretLeft);
        assert_eq!(snapshot.selection_anchor, 1);

        let snapshot = runtime.dispatch(EditorEvent::Backspace);
        assert_eq!(snapshot.text, "a");
        assert_eq!(snapshot.selection_anchor, 0);
    }

    #[test]
    fn test_composition_preview_does_not_commit_document() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::CompositionStart);
        let snapshot = runtime.dispatch(EditorEvent::CompositionUpdate {
            text: "你".to_string(),
        });

        assert_eq!(snapshot.text, "");
        assert_eq!(snapshot.display_text, "你");
        assert_eq!(
            snapshot.composition,
            Some(CompositionSnapshot {
                from: 0,
                to: 0,
                text: "你".to_string(),
            })
        );
        assert_eq!(snapshot.scene.composition_underlines.len(), 1);
        assert_eq!(snapshot.revision, 0);
    }

    #[test]
    fn test_composition_end_commits_text() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::CompositionStart);
        runtime.dispatch(EditorEvent::CompositionUpdate {
            text: "你".to_string(),
        });
        let snapshot = runtime.dispatch(EditorEvent::CompositionEnd {
            text: "你".to_string(),
        });

        assert_eq!(snapshot.text, "你");
        assert_eq!(snapshot.display_text, "你");
        assert_eq!(snapshot.composition, None);
        assert_eq!(snapshot.revision, 1);
        assert_eq!(snapshot.selection_anchor, 1);
    }

    #[test]
    fn test_text_measurements_change_wrap_width() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::ResizeViewport {
            width: 24.0 * 2.0 + 25.0,
            height: 480.0,
            device_pixel_ratio: 1.0,
        });
        let snapshot = runtime.dispatch(EditorEvent::InsertText {
            text: "你a".to_string(),
        });
        assert_eq!(snapshot.lines.len(), 1);

        let snapshot = runtime.dispatch(EditorEvent::SetTextMeasurements {
            entries: vec![TextMeasurementEntry {
                style_key: "text.primary".to_string(),
                text: "你".to_string(),
                advance: 20.0,
            }],
        });
        assert_eq!(snapshot.lines.len(), 2);
    }

    #[test]
    fn test_vertical_movement_uses_visual_lines() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::ResizeViewport {
            width: 24.0 * 2.0 + 3.0 * 9.6,
            height: 480.0,
            device_pixel_ratio: 1.0,
        });
        runtime.dispatch(EditorEvent::InsertText {
            text: "abcdef".to_string(),
        });
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
        let snapshot = runtime.dispatch(EditorEvent::InsertText {
            text: "abc\ndef".to_string(),
        });

        assert!(snapshot.content_width >= 48.0);
        assert!(snapshot.content_height >= 112.0);
        assert_eq!(snapshot.lines.len(), 2);
    }

    #[test]
    fn test_inserting_newline_creates_multiple_paragraphs() {
        let mut runtime = EditorRuntime::new();
        runtime.dispatch(EditorEvent::InsertText {
            text: "abc\ndef".to_string(),
        });

        assert_eq!(
            runtime.state.doc().paragraph_texts(),
            vec!["abc".to_string(), "def".to_string()]
        );
        assert_eq!(runtime.state.doc().plain_text(), "abc\ndef");
    }
}
