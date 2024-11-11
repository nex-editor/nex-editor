#[derive(Debug)]
pub struct Page {
    height: u32,
    width: u32,
    name: &'static str,
    padding: Padding,
}

impl Page {
    pub(crate) fn new() -> Page {
        Page {
            height: 0,
            width: 0,
            name: "",
            padding: Padding {
                top: 0,
                bottom: 0,
                left: 0,
                right: 0
            }
        }
    }
}

#[derive(Debug)]
pub struct Padding {
    top: u32,
    bottom: u32,
    left: u32,
    right: u32
}

pub const A4: Page = Page {
    height: 1122,
    width: 793,
    name: "A4",
    padding: Padding {
        top: 56,
        bottom: 56,
        left: 56,
        right: 56
    }
};