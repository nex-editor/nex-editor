use super::*;

impl EditorRuntime {
    pub(crate) fn move_caret_vertical(&mut self, direction: isize) {
        let text_layout = self.text_layout(&self.state.doc().plain_text());
        let selection = self.state.selection();
        let offset = FlatTextOffset::new(if selection.is_collapsed() {
            selection.head()
        } else if direction < 0 {
            selection.from()
        } else {
            selection.to()
        });

        let Some(current_line_index) = self.line_index_for_offset(&text_layout, offset) else {
            return;
        };
        let target_line_index = current_line_index as isize + direction;
        if target_line_index < 0 || target_line_index as usize >= text_layout.lines.len() {
            return;
        }

        let current_line = &text_layout.lines[current_line_index];
        let target_line = &text_layout.lines[target_line_index as usize];
        let visual_column = offset.get().saturating_sub(current_line.start);
        let next = FlatTextOffset::new(
            target_line.start
                + visual_column.min(target_line.end.saturating_sub(target_line.start)),
        );
        self.set_selection(next, next);
    }
}
