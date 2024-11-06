use tiny_skia as sk;
const SCALE: f32 = 3.0;

pub fn skia_render() {
    // A4
    let width = 793.0;
    let height = 1122.0;
    let mut canvas = sk::Pixmap::new((width * SCALE) as u32, (height * SCALE) as u32).unwrap();
    // fill page
    canvas.fill(sk::Color::WHITE);

    let font = include_bytes!("../../assets/PingFang.ttc") as &[u8];
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    let (metrics, bitmap) = font.rasterize('A', 16.0 * SCALE);

    println!("{:?}", metrics);

    let mut text_pixmap = sk::Pixmap::new(metrics.width as u32, metrics.height as u32).unwrap();
    for x in 0..metrics.width {
        for y in 0..metrics.height {
            let pixel = bitmap[(x + y * metrics.width) as usize];
            if pixel > 0 {
                text_pixmap.pixels_mut()[y as usize * metrics.width as usize + x as usize] = sk::ColorU8::from_rgba(
                    0,
                    0,
                    0,
                    pixel
                ).premultiply();
            }
        }
    }

    canvas.draw_pixmap(
        0,
        0,
        text_pixmap.as_ref(),
        &sk::PixmapPaint::default(),
        sk::Transform::identity(),
        None
    );

    // save to file
    canvas.save_png("./target/test.png").unwrap();
}