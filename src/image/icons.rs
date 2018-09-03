use super::png_loader::PngImage;
use std::fs::File;

lazy_static! {
    pub static ref TOOL_POINT: PngImage =
        PngImage::load(File::open("resources/tool_point.png").unwrap()).unwrap();
    pub static ref TOOL_LINE: PngImage =
        PngImage::load(File::open("resources/tool_line.png").unwrap()).unwrap();
    pub static ref TOOL_CIRCLE: PngImage =
        PngImage::load(File::open("resources/tool_circle.png").unwrap()).unwrap();
    pub static ref TOOL_MOVER: PngImage =
        PngImage::load(File::open("resources/tool_mover.png").unwrap()).unwrap();
}
