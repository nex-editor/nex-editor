use tiny_skia::Pixmap;
use tiny_skia as sk;

pub fn skia_render(font: &fontdue::Font, canvas: &mut Pixmap, scale: f32) {
    let (metrics, bitmap) = font.rasterize('A', 16.0 * 1.0);
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
    // canvas.save_png("./target/test.png").unwrap();
}