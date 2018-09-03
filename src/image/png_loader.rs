extern crate sdl2;

use png::{ColorType, Decoder, DecodingError};

use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;

use std::io::Read;

#[derive(Clone)]
pub struct PngImage {
    pub width: usize,
    pub height: usize,
    /// The data in the image, stored in chunks of 4 per pixel, containing the image in ABGR order
    pub data: Vec<u8>,
}

impl PngImage {
    /// Load an image from a specified source.
    pub fn load<R: Read>(r: R) -> Result<Self, DecodingError> {
        let (info, mut reader) = Decoder::new(r).read_info()?;

        let (width, height) = (info.width as usize, info.height as usize);

        let mut data = vec![0; width * height * 4];

        for y in 0..height {
            if let Some(row) = reader.next_row()? {
                assert_eq!(row.len(), width * info.color_type.samples());

                for (x, col) in row.chunks(info.color_type.samples()).enumerate() {
                    let sdl_col = match info.color_type {
                        ColorType::RGB => Color::RGB(col[0], col[1], col[2]),
                        ColorType::RGBA => Color::RGBA(col[0], col[1], col[2], col[3]),
                        _ => unimplemented!(),
                    };

                    data[(y * width + x) * 4] = sdl_col.b;
                    data[(y * width + x) * 4 + 1] = sdl_col.g;
                    data[(y * width + x) * 4 + 2] = sdl_col.r;
                    data[(y * width + x) * 4 + 3] = sdl_col.a;
                }
            }
        }

        Ok(PngImage {
            width,
            height,
            data,
        })
    }

    pub fn draw(&self, canvas: &mut Canvas<Window>, pos: Point) -> Result<(), String> {
        let creator = canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(None, self.width as u32, self.height as u32)
            .map_err(|e| format!("TextureValue error: {:?}", e))?;

        texture.set_blend_mode(BlendMode::Blend);

        texture
            .update(None, self.data.as_slice(), 4 * self.width)
            .map_err(|e| format!("UpdateTexture error: {:?}", e))?;

        let rect = Rect::new(pos.x(), pos.y(), self.width as u32, self.height as u32);

        canvas.copy(&texture, None, rect)
    }
}
