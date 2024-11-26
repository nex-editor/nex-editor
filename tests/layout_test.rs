use nex_editor::layout::layout_frame;
use sample::sample1;

pub mod sample;

#[test]
pub fn test_layout_frame() {
    let (mut editor, string) = sample1();
    layout_frame(&mut editor.state);
}