use getset::{CopyGetters, Getters};
use png::{Decoder, Transformations};
use std::fs::File;
use std::io::BufReader;

use dear_imgui_reflect::ImGuiReflect;

use crate::common::asset_path;

#[derive(CopyGetters, Getters, Clone)]
pub struct Texture {
    #[getset(get = "pub with_prefix")]
    image_data: Vec<u8>,
    #[getset(get = "pub with_prefix")]
    image_dimensions: [u32; 2],
}

impl Texture {
    pub fn new(path: impl AsRef<str>) -> Self {
        let file = File::open(asset_path(path.as_ref())).unwrap();

        let mut decoder = Decoder::new(BufReader::new(file));
        decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
        let mut reader = decoder.read_info().unwrap();

        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let frame = reader.next_frame(&mut buf).unwrap();
        buf.truncate(frame.buffer_size());

        let info = reader.info();

        Self {
            image_data: buf,
            image_dimensions: [info.width, info.height],
        }
    }

    pub fn from_sprite_sheet(
        path: impl AsRef<str>,
        sprite_index: [u32; 2],
        sprite_size: [u32; 2],
    ) -> Self {
        let file = File::open(asset_path(path.as_ref())).unwrap();

        let mut decoder = Decoder::new(BufReader::new(file));
        decoder.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
        let mut reader = decoder.read_info().unwrap();

        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let frame = reader.next_frame(&mut buf).unwrap();

        let stride = 4usize;
        let sheet_width = frame.width as usize;
        let (sprite_width, sprite_height) = (sprite_size[0] as usize, sprite_size[1] as usize);
        let (sprite_x, sprite_y) = (
            sprite_index[0] as usize * sprite_width,
            sprite_index[1] as usize * sprite_height,
        );

        let mut image_data: Vec<u8> = Vec::with_capacity(stride * (sprite_width * sprite_height));

        for row in 0..sprite_height {
            let start_pixel_idx = ((sprite_y + row) * sheet_width) + sprite_x;
            let end_pixel_idx = start_pixel_idx + sprite_width;

            let start_byte = start_pixel_idx * stride;
            let end_byte = end_pixel_idx * stride;

            image_data.extend_from_slice(&buf[start_byte..end_byte]);
        }

        Self {
            image_data,
            image_dimensions: sprite_size,
        }
    }
}
