use std::io::Cursor;
use ytesrev::image::PngImage;

macro_rules! load_image {
    ($path:expr) => {
        PngImage::load_from_path(Cursor::new(
            include_bytes!(concat!("../../resources/", $path)) as &[u8],
        )).unwrap()
    };
}

lazy_static! {
    pub static ref TOOL_POINT: PngImage = load_image!("tool_point.png");
    pub static ref TOOL_LINE: PngImage = load_image!("tool_line.png");
    pub static ref TOOL_CIRCLE: PngImage = load_image!("tool_circle.png");
    pub static ref TOOL_MOVER: PngImage = load_image!("tool_mover.png");
    pub static ref TOOL_SELECTOR: PngImage = load_image!("tool_selector.png");

    pub static ref CIRCLE_NORMAL: PngImage = load_image!("circle_normal.png");
    pub static ref CIRCLE_SELECT: PngImage = load_image!("circle_select.png");
    pub static ref CIRCLE_MOVING: PngImage = load_image!("circle_moving.png");
    pub static ref CIRCLE_MOVER: PngImage = load_image!("circle_mover.png");
}
