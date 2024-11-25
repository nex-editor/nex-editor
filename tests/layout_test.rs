use sample::sample1;

pub mod sample;

#[test]
pub fn test_layout_frame() {
    let (editor, string) = sample1();
    println!("{:?}", editor);
}