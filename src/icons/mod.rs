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
    pub static ref TOOL_PEEK: PngImage = load_image!("tool_peek.png");
    pub static ref SELECTED_HIDE: PngImage = load_image!("selected_hide.png");
    pub static ref SELECTED_SHOW: PngImage = load_image!("selected_show.png");
    pub static ref CIRCLE_NORMAL: CircleImage = CircleImage::new(load_image!("circle_normal.png"));
    pub static ref CIRCLE_PRIMARY: CircleImage =
        CircleImage::new(load_image!("circle_primary.png"));
    pub static ref CIRCLE_ACTIVE: CircleImage = CircleImage::new(load_image!("circle_active.png"));
}

pub struct CircleImage {
    pub normal: PngImage,
    pub transparent: PngImage,
}

impl CircleImage {
    pub fn new(normal: PngImage) -> CircleImage {
        // Make transparent
        let mut transparent = normal.clone();

        for i in 0..transparent.data.len() / 4 {
            transparent.data[i * 4 + 3] /= 4;
        }

        CircleImage {
            normal,
            transparent,
        }
    }
}
