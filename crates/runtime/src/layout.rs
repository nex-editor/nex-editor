use super::*;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
struct LayoutUnit {
    text: String,
    start: usize,
    end: usize,
    advance: f32,
}

impl EditorRuntime {
    pub(crate) fn scene_snapshot(
        &self,
        text_layout: &TextLayout,
        selection_rects: &[SelectionRect],
        composition_underlines: &[SelectionRect],
        caret: Option<CaretLayout>,
    ) -> SceneSnapshot {
        SceneSnapshot {
            viewport: self.viewport,
            content_width: text_layout.content_width,
            content_height: text_layout.content_height,
            styles: vec![
                PaintStyle {
                    id: "surface".to_string(),
                    role: PaintStyleRole::EditorSurface,
                    measurement_style_key: None,
                },
                PaintStyle {
                    id: "text.primary".to_string(),
                    role: PaintStyleRole::PrimaryText,
                    measurement_style_key: Some(self.layout.text_style_key.clone()),
                },
                PaintStyle {
                    id: "selection.fill".to_string(),
                    role: PaintStyleRole::SelectionFill,
                    measurement_style_key: None,
                },
                PaintStyle {
                    id: "caret.fill".to_string(),
                    role: PaintStyleRole::CaretFill,
                    measurement_style_key: None,
                },
                PaintStyle {
                    id: "composition.underline".to_string(),
                    role: PaintStyleRole::CompositionUnderline,
                    measurement_style_key: None,
                },
            ],
            background: vec![PaintRect {
                kind: PaintRectKind::Background,
                style_id: "surface".to_string(),
                x: 0.0,
                y: 0.0,
                width: self.viewport.width,
                height: self.viewport.height,
            }],
            selection_rects: selection_rects
                .iter()
                .copied()
                .map(|rect| PaintRect {
                    kind: PaintRectKind::Selection,
                    style_id: "selection.fill".to_string(),
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                })
                .collect(),
            composition_underlines: composition_underlines
                .iter()
                .copied()
                .map(|rect| PaintRect {
                    kind: PaintRectKind::Selection,
                    style_id: "composition.underline".to_string(),
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: rect.height,
                })
                .collect(),
            text_runs: text_layout
                .lines
                .iter()
                .flat_map(|line| line.runs.iter())
                .map(|run| PaintTextRun {
                    text: run.text.clone(),
                    style_id: "text.primary".to_string(),
                    x: run.x,
                    baseline_y: run.baseline_y,
                    width: run.width,
                    height: run.height,
                })
                .collect(),
            caret: caret.map(|value| PaintRect {
                kind: PaintRectKind::Caret,
                style_id: "caret.fill".to_string(),
                x: value.x,
                y: value.y,
                width: value.width,
                height: value.height,
            }),
        }
    }

    pub(crate) fn composition_underlines(
        &self,
        text_layout: &TextLayout,
        range: Option<(usize, usize)>,
    ) -> Vec<SelectionRect> {
        let Some((from, to)) = range else {
            return Vec::new();
        };
        if from == to {
            return Vec::new();
        }

        let mut rects = Vec::new();
        for line in &text_layout.lines {
            let start = from.max(line.start);
            let end = to.min(line.end);
            if end <= start {
                continue;
            }

            rects.push(SelectionRect {
                x: self.x_for_offset_in_line(line, start),
                y: line.baseline_y + self.layout.descent.max(1.0) * 0.25,
                width: self.x_for_offset_in_line(line, end)
                    - self.x_for_offset_in_line(line, start),
                height: 2.0,
            });
        }

        rects
    }

    pub(crate) fn text_layout(&self, text: &str) -> TextLayout {
        let max_width =
            (self.viewport.width - self.layout.padding_x * 2.0).max(self.layout.char_width);
        let mut lines = Vec::new();
        let mut line_units: Vec<LayoutUnit> = Vec::new();
        let mut line_width = 0.0;
        let mut line_start = 0usize;
        let mut global_index = 0usize;
        let mut line_index = 0usize;

        let push_line = |lines: &mut Vec<LineLayout>,
                         line_units: &[LayoutUnit],
                         line_width: f32,
                         line_start: usize,
                         line_end: usize,
                         line_index: usize,
                         layout: &LayoutConfig| {
            let y = layout.padding_y + line_index as f32 * layout.line_height;
            let baseline_y = y + layout.ascent;
            let mut x = layout.padding_x;
            let runs = line_units
                .iter()
                .map(|unit| {
                    let run = TextRun {
                        text: unit.text.clone(),
                        start: unit.start,
                        end: unit.end,
                        x,
                        y,
                        baseline_y,
                        width: unit.advance,
                        height: layout.line_height,
                    };
                    x += unit.advance;
                    run
                })
                .collect();
            lines.push(LineLayout {
                line_index,
                start: line_start,
                end: line_end,
                x: layout.padding_x,
                y,
                baseline_y,
                width: line_width,
                height: layout.line_height,
                runs,
            });
        };

        for unit_text in UnicodeSegmentation::graphemes(text, true) {
            let char_len = Node::char_len(unit_text);
            if unit_text == "\n" {
                push_line(
                    &mut lines,
                    &line_units,
                    line_width,
                    line_start,
                    global_index,
                    line_index,
                    &self.layout,
                );
                line_units.clear();
                line_width = 0.0;
                global_index += 1;
                line_start = global_index;
                line_index += 1;
                continue;
            }

            let advance = self.advance_for_text(unit_text);
            if !line_units.is_empty() && line_width + advance > max_width {
                push_line(
                    &mut lines,
                    &line_units,
                    line_width,
                    line_start,
                    global_index,
                    line_index,
                    &self.layout,
                );
                line_units.clear();
                line_width = 0.0;
                line_start = global_index;
                line_index += 1;
            }

            let start = global_index;
            global_index += char_len;
            line_units.push(LayoutUnit {
                text: unit_text.to_string(),
                start,
                end: global_index,
                advance,
            });
            line_width += advance;
        }

        push_line(
            &mut lines,
            &line_units,
            line_width,
            line_start,
            global_index,
            line_index,
            &self.layout,
        );
        let content_width = lines
            .iter()
            .map(|line| line.x + line.width + self.layout.padding_x)
            .fold(self.layout.padding_x * 2.0, f32::max);
        let content_height = lines
            .last()
            .map(|line| line.y + line.height + self.layout.padding_y)
            .unwrap_or(self.layout.padding_y * 2.0);

        TextLayout {
            lines,
            content_width,
            content_height,
        }
    }

    pub(crate) fn hit_test(&self, text_layout: &TextLayout, x: f32, y: f32) -> HitTestResult {
        let row = ((y - self.layout.padding_y + self.layout.line_height / 2.0)
            / self.layout.line_height)
            .floor()
            .max(0.0) as usize;
        let line = text_layout
            .lines
            .get(row)
            .or_else(|| text_layout.lines.last())
            .expect("layout always has at least one line");
        HitTestResult {
            offset: FlatTextOffset::new(self.offset_for_line_x(line, x)),
        }
    }

    pub(crate) fn line_index_for_offset(
        &self,
        text_layout: &TextLayout,
        offset: FlatTextOffset,
    ) -> Option<usize> {
        let offset = offset.get();
        let mut last_match = None;
        for (index, line) in text_layout.lines.iter().enumerate() {
            if offset >= line.start && offset <= line.end {
                return Some(index);
            }
            if offset > line.end {
                last_match = Some(index);
            }
        }
        last_match.or_else(|| (!text_layout.lines.is_empty()).then_some(0))
    }

    pub(crate) fn selection_rects(
        &self,
        text_layout: &TextLayout,
        anchor: usize,
        head: usize,
    ) -> Vec<SelectionRect> {
        let from = anchor.min(head);
        let to = anchor.max(head);
        if from == to {
            return Vec::new();
        }

        let mut rects = Vec::new();
        for line in &text_layout.lines {
            let start = from.max(line.start);
            let end = to.min(line.end);
            if end <= start {
                continue;
            }

            rects.push(SelectionRect {
                x: self.x_for_offset_in_line(line, start),
                y: line.baseline_y - self.layout.ascent,
                width: self.x_for_offset_in_line(line, end)
                    - self.x_for_offset_in_line(line, start),
                height: self.layout.ascent + self.layout.descent,
            });
        }

        rects
    }

    pub(crate) fn caret_layout(
        &self,
        text_layout: &TextLayout,
        anchor: usize,
        head: usize,
    ) -> Option<CaretLayout> {
        if anchor != head {
            return None;
        }

        let offset = head;
        let mut line = text_layout.lines.first()?;
        for candidate in &text_layout.lines {
            if offset >= candidate.start && offset <= candidate.end {
                line = candidate;
                break;
            }
            if offset > candidate.end {
                line = candidate;
            }
        }

        Some(CaretLayout {
            x: self.x_for_offset_in_line(line, offset),
            y: line.baseline_y - self.layout.ascent,
            width: self.layout.caret_width,
            height: self.layout.ascent + self.layout.descent,
        })
    }

    pub(crate) fn x_for_offset_in_line(&self, line: &LineLayout, offset: usize) -> f32 {
        if offset <= line.start {
            return line.x;
        }

        let mut current_x = line.x;
        for run in &line.runs {
            if offset >= run.end {
                current_x = run.x + run.width;
                continue;
            }

            if offset <= run.start {
                return run.x;
            }

            let unit_len = (run.end - run.start).max(1);
            let consumed = offset.saturating_sub(run.start).min(unit_len);
            return run.x + run.width * (consumed as f32 / unit_len as f32);
        }

        current_x
    }

    pub(crate) fn offset_for_line_x(&self, line: &LineLayout, x: f32) -> usize {
        if x <= line.x {
            return line.start;
        }

        let mut best_offset = line.end;
        let mut best_distance = f32::INFINITY;

        for run in &line.runs {
            let unit_len = (run.end - run.start).max(1);
            for index in 0..=unit_len {
                let boundary_x = run.x + run.width * (index as f32 / unit_len as f32);
                let distance = (x - boundary_x).abs();
                if distance < best_distance {
                    best_distance = distance;
                    best_offset = run.start + index;
                }
            }
        }

        best_offset
    }

    pub(crate) fn advance_for_text(&self, text: &str) -> f32 {
        self.measurements
            .get(&self.layout.text_style_key)
            .and_then(|entries| entries.get(text))
            .copied()
            .unwrap_or_else(|| self.layout.char_width * Node::char_len(text) as f32)
    }
}
