use super::*;

impl EditorRuntime {
    pub(crate) fn pointer_down(
        &mut self,
        x: f32,
        y: f32,
        button: PointerButton,
    ) {
        if button != PointerButton::Primary {
            return;
        }

        let text_layout = self.text_layout(&self.state.doc().plain_text());
        let hit = self.hit_test(&text_layout, x, y);
        self.pointer_anchor = Some(hit.offset);
        self.set_selection(hit.offset, hit.offset);
    }

    pub(crate) fn pointer_move(&mut self, x: f32, y: f32) {
        if let Some(anchor) = self.pointer_anchor {
            let text_layout = self.text_layout(&self.state.doc().plain_text());
            let hit = self.hit_test(&text_layout, x, y);
            self.set_selection(anchor, hit.offset);
        }
    }

    pub(crate) fn pointer_up(&mut self) {
        self.pointer_anchor = None;
    }
}
