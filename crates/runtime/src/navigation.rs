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
        let target_x = self.x_for_offset_in_line(current_line, offset.get());
        let next = FlatTextOffset::new(self.offset_for_line_x(target_line, target_x));
        self.set_selection(next, next);
    }
}
