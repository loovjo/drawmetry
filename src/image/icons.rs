use super::png_loader::PngImage;
use std::io::Cursor;

macro_rules! load_image {
    ($path:expr) => {
        PngImage::load(Cursor::new(include_bytes!(concat!("../../resources/", $path)) as &[u8])).unwrap()
    };
}

lazy_static! {
    pub static ref TOOL_POINT: PngImage = load_image!("tool_point.png");
    pub static ref TOOL_LINE: PngImage = load_image!("tool_line.png");
    pub static ref TOOL_CIRCLE: PngImage = load_image!("tool_circle.png");
    pub static ref TOOL_MOVER: PngImage = load_image!("tool_mover.png");
}
