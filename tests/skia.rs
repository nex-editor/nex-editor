use nex_editor::render::skia::skia_render;
use tiny_skia as sk;

#[test]
fn test_skia() {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    const FAMILY_NAME: &str = "Sans Serif";
    let query = fontdb::Query {
        families: &[fontdb::Family::Name(FAMILY_NAME), fontdb::Family::Serif],
        weight: fontdb::Weight::BOLD,
        ..fontdb::Query::default()
    };
    let id = db.query(&query).unwrap();
    db.with_face_data(id, |font, _| {
        const SCALE: f32 = 3.0;
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        // A4
        let width = 793.0;
        let height = 1122.0;
        let mut canvas = sk::Pixmap::new((width * SCALE) as u32, (height * SCALE) as u32).unwrap();
        canvas.fill(sk::Color::WHITE);
        skia_render(&font, &mut canvas, 3.0)
    });
}