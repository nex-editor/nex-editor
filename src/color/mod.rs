#[derive(Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl Color {
    pub const BLACK: Self = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Self = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Self = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Self = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Self = Color { r: 0, g: 0, b: 255, a: 255 };
    pub const TRANSPARENT: Self = Color { r: 0, g: 0, b: 0, a: 0 };
}
