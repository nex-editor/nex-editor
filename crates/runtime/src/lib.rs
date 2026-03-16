//! Headless editor runtime with layout and hit-testing owned by Rust.

use model::Node;
use serde::{Deserialize, Serialize};
use state::{EditorState, Selection, TextSelection};

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
    pointer_anchor: Option<usize>,
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
        let text = self.state.doc().text_content();
        let lines = self.layout_text(&text);
        let selection_anchor = self.state.selection().anchor();
        let selection_head = self.state.selection().head();
        let content_width = lines
            .iter()
            .map(|line| line.x + line.width + self.layout.padding_x)
            .fold(self.layout.padding_x * 2.0, f32::max);
        let content_height = lines
            .last()
            .map(|line| line.y + line.height + self.layout.padding_y)
            .unwrap_or(self.layout.padding_y * 2.0);

        RenderSnapshot {
            text,
            selection_anchor,
            selection_head,
            revision: self.revision,
            viewport: self.viewport,
            content_width,
            content_height,
            selection_rects: self.selection_rects(&lines, selection_anchor, selection_head),
            caret: self.caret_layout(&lines, selection_anchor, selection_head),
            lines,
        }
    }

    pub fn dispatch(&mut self, event: EditorEvent) -> RenderSnapshot {
        match event {
            EditorEvent::ResizeViewport { width, height, device_pixel_ratio } => {
                self.viewport = Viewport { width, height, device_pixel_ratio };
            }
            EditorEvent::PointerDown { x, y, button: PointerButton::Primary, .. } => {
                let offset = self.hit_test_offset(x, y);
                self.pointer_anchor = Some(offset);
                self.set_selection(offset, offset);
            }
            EditorEvent::PointerDown { .. } => {}
            EditorEvent::PointerMove { x, y, .. } => {
                if let Some(anchor) = self.pointer_anchor {
                    let head = self.hit_test_offset(x, y);
                    self.set_selection(anchor, head);
                }
            }
            EditorEvent::PointerUp { .. } => {
                self.pointer_anchor = None;
            }
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

    fn set_selection(&mut self, anchor: usize, head: usize) {
        let text_len = self.state.doc().text_content().len();
        let anchor = anchor.min(text_len);
        let head = head.min(text_len);
        let selection = Selection::text(TextSelection::create(self.state.doc(), anchor, head));
        self.state = self.state.with_selection(selection);
    }

    fn insert_text(&mut self, text: String) {
        let selection = self.state.selection();
        let from = selection.from();
        let to = selection.to();
        let mut next = self.state.doc().text_content();
        next.replace_range(from..to, &text);
        let cursor = from + text.len();
        self.apply_text_edit(next, cursor, cursor);
    }

    fn backspace(&mut self) {
        let selection = self.state.selection();
        let from = selection.from();
        let to = selection.to();
        if from != to {
            let mut next = self.state.doc().text_content();
            next.replace_range(from..to, "");
            self.apply_text_edit(next, from, from);
            return;
        }

        if from == 0 {
            return;
        }

        let mut next = self.state.doc().text_content();
        next.replace_range((from - 1)..from, "");
        self.apply_text_edit(next, from - 1, from - 1);
    }

    fn delete_forward(&mut self) {
        let selection = self.state.selection();
        let from = selection.from();
        let to = selection.to();
        if from != to {
            let mut next = self.state.doc().text_content();
            next.replace_range(from..to, "");
            self.apply_text_edit(next, from, from);
            return;
        }

        let text_len = self.state.doc().text_content().len();
        if from >= text_len {
            return;
        }

        let mut next = self.state.doc().text_content();
        next.replace_range(from..(from + 1), "");
        self.apply_text_edit(next, from, from);
    }

    fn move_caret_left(&mut self) {
        let selection = self.state.selection();
        if !selection.is_collapsed() {
            let next = selection.from();
            self.set_selection(next, next);
            return;
        }
        let next = selection.head().saturating_sub(1);
        self.set_selection(next, next);
    }

    fn move_caret_right(&mut self) {
        let selection = self.state.selection();
        let text_len = self.state.doc().text_content().len();
        if !selection.is_collapsed() {
            let next = selection.to();
            self.set_selection(next, next);
            return;
        }
        let next = (selection.head() + 1).min(text_len);
        self.set_selection(next, next);
    }

    fn select_all(&mut self) {
        let text_len = self.state.doc().text_content().len();
        self.set_selection(0, text_len);
    }

    fn move_caret_vertical(&mut self, direction: isize) {
        let snapshot = self.snapshot();
        let selection = self.state.selection();
        let offset = if selection.is_collapsed() {
            selection.head()
        } else if direction < 0 {
            selection.from()
        } else {
            selection.to()
        };

        let Some(current_line_index) = self.line_index_for_offset(&snapshot.lines, offset) else {
            return;
        };
        let target_line_index = current_line_index as isize + direction;
        if target_line_index < 0 || target_line_index as usize >= snapshot.lines.len() {
            return;
        }

        let current_line = &snapshot.lines[current_line_index];
        let target_line = &snapshot.lines[target_line_index as usize];
        let visual_column = offset.saturating_sub(current_line.start);
        let next = target_line.start + visual_column.min(target_line.end.saturating_sub(target_line.start));
        self.set_selection(next, next);
    }

    fn state_from_text(&self, text: String, anchor: usize, head: usize) -> EditorState {
        let doc = Node::from_paragraph_texts(vec![text]);
        let selection = Selection::text(TextSelection::create(&doc, anchor, head));
        EditorState::new(doc, selection, self.state.schema().clone())
    }

    fn apply_text_edit(&mut self, next_text: String, anchor: usize, head: usize) {
        if self.state.doc().text_content() == next_text {
            return;
        }
        self.state = self.state_from_text(next_text, anchor, head);
        self.revision += 1;
    }

    fn layout_text(&self, text: &str) -> Vec<LineLayout> {
        let max_columns = ((self.viewport.width - self.layout.padding_x * 2.0) / self.layout.char_width)
            .floor()
            .max(1.0) as usize;

        let mut lines = Vec::new();
        let mut line_text = String::new();
        let mut line_start = 0usize;
        let mut global_index = 0usize;
        let mut line_index = 0usize;

        let push_line = |lines: &mut Vec<LineLayout>, line_text: &str, line_start: usize, line_end: usize, line_index: usize, layout: LayoutConfig| {
            let width = line_text.chars().count() as f32 * layout.char_width;
            let y = layout.padding_y + line_index as f32 * layout.line_height;
            lines.push(LineLayout {
                line_index,
                start: line_start,
                end: line_end,
                x: layout.padding_x,
                y,
                width,
                height: layout.line_height,
                runs: vec![TextRun {
                    text: line_text.to_string(),
                    start: line_start,
                    end: line_end,
                    x: layout.padding_x,
                    y,
                    width,
                    height: layout.line_height,
                }],
            });
        };

        for ch in text.chars() {
            if ch == '\n' {
                push_line(&mut lines, &line_text, line_start, global_index, line_index, self.layout);
                line_text.clear();
                global_index += 1;
                line_start = global_index;
                line_index += 1;
                continue;
            }

            if line_text.chars().count() >= max_columns {
                push_line(&mut lines, &line_text, line_start, global_index, line_index, self.layout);
                line_text.clear();
                line_start = global_index;
                line_index += 1;
            }

            line_text.push(ch);
            global_index += ch.len_utf8();
        }

        push_line(&mut lines, &line_text, line_start, global_index, line_index, self.layout);
        lines
    }

    fn hit_test_offset(&self, x: f32, y: f32) -> usize {
        let snapshot = self.snapshot();
        let row = ((y - self.layout.padding_y + self.layout.line_height / 2.0) / self.layout.line_height)
            .floor()
            .max(0.0) as usize;
        let line = snapshot
            .lines
            .get(row)
            .or_else(|| snapshot.lines.last())
            .expect("layout always has at least one line");
        let local_x = (x - self.layout.padding_x).max(0.0);
        let char_index = (local_x / self.layout.char_width).round().max(0.0) as usize;
        line.start + char_index.min(line.end.saturating_sub(line.start))
    }

    fn line_index_for_offset(&self, lines: &[LineLayout], offset: usize) -> Option<usize> {
        let mut last_match = None;
        for (index, line) in lines.iter().enumerate() {
            if offset >= line.start && offset <= line.end {
                return Some(index);
            }
            if offset > line.end {
                last_match = Some(index);
            }
        }
        last_match.or_else(|| (!lines.is_empty()).then_some(0))
    }

    fn selection_rects(&self, lines: &[LineLayout], anchor: usize, head: usize) -> Vec<SelectionRect> {
        let from = anchor.min(head);
        let to = anchor.max(head);
        if from == to {
            return Vec::new();
        }

        let mut rects = Vec::new();
        for line in lines {
            let start = from.max(line.start);
            let end = to.min(line.end);
            if end <= start {
                continue;
            }

            rects.push(SelectionRect {
                x: self.layout.padding_x + (start - line.start) as f32 * self.layout.char_width,
                y: line.y - 2.0,
                width: (end - start) as f32 * self.layout.char_width,
                height: self.layout.line_height - 4.0,
            });
        }

        rects
    }

    fn caret_layout(&self, lines: &[LineLayout], anchor: usize, head: usize) -> Option<CaretLayout> {
        if anchor != head {
            return None;
        }

        let offset = head;
        let mut line = lines.first()?;
        for candidate in lines {
            if offset >= candidate.start && offset <= candidate.end {
                line = candidate;
                break;
            }
            if offset > candidate.end {
                line = candidate;
            }
        }

        Some(CaretLayout {
            x: self.layout.padding_x + (offset - line.start) as f32 * self.layout.char_width,
            y: line.y - 1.0,
            width: self.layout.caret_width,
            height: self.layout.line_height - 6.0,
        })
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
}
