use nex_editor::render::skia::skia_render;
use criterion::{criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();
    const FAMILY_NAME: &str = "PingFang SC";
    let query = fontdb::Query {
        families: &[fontdb::Family::Name(FAMILY_NAME), fontdb::Family::SansSerif],
        weight: fontdb::Weight::BOLD,
        ..fontdb::Query::default()
    };
    let id = db.query(&query).unwrap();
    db.with_face_data(id, |font, _| {
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
        c.bench_function("skia_render 10", |b| b.iter(|| skia_render(&font)));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);