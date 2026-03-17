use super::*;

impl EditorRuntime {
    pub(crate) fn text_layout(&self, text: &str) -> TextLayout {
        let max_columns =
            ((self.viewport.width - self.layout.padding_x * 2.0) / self.layout.char_width)
                .floor()
                .max(1.0) as usize;

        let mut lines = Vec::new();
        let mut line_text = String::new();
        let mut line_start = 0usize;
        let mut global_index = 0usize;
        let mut line_index = 0usize;

        let push_line = |lines: &mut Vec<LineLayout>,
                         line_text: &str,
                         line_start: usize,
                         line_end: usize,
                         line_index: usize,
                         layout: LayoutConfig| {
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
                push_line(
                    &mut lines,
                    &line_text,
                    line_start,
                    global_index,
                    line_index,
                    self.layout,
                );
                line_text.clear();
                global_index += 1;
                line_start = global_index;
                line_index += 1;
                continue;
            }

            if line_text.chars().count() >= max_columns {
                push_line(
                    &mut lines,
                    &line_text,
                    line_start,
                    global_index,
                    line_index,
                    self.layout,
                );
                line_text.clear();
                line_start = global_index;
                line_index += 1;
            }

            line_text.push(ch);
            global_index += ch.len_utf8();
        }

        push_line(
            &mut lines,
            &line_text,
            line_start,
            global_index,
            line_index,
            self.layout,
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
        let local_x = (x - self.layout.padding_x).max(0.0);
        let char_index = (local_x / self.layout.char_width).round().max(0.0) as usize;
        HitTestResult {
            offset: FlatTextOffset::new(
                line.start + char_index.min(line.end.saturating_sub(line.start)),
            ),
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
                x: self.layout.padding_x + (start - line.start) as f32 * self.layout.char_width,
                y: line.y - 2.0,
                width: (end - start) as f32 * self.layout.char_width,
                height: self.layout.line_height - 4.0,
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
            x: self.layout.padding_x + (offset - line.start) as f32 * self.layout.char_width,
            y: line.y - 1.0,
            width: self.layout.caret_width,
            height: self.layout.line_height - 6.0,
        })
    }
}
