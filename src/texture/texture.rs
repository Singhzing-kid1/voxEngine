use getset::{CopyGetters, Getters};
use png::Decoder;
use std::fs::File;

#[derive(CopyGetters, Getters, Clone)]
pub struct Texture {
    #[getset(get = "pub with_prefix")]
    image_data: Vec<u8>,
    #[getset(get = "pub with_prefix")]
    image_dimensions: [u32; 2],
}

impl Texture {
    pub fn new(path: impl Into<String>) -> Self {
        let mut image_data = Vec::new();

        let base = env!("CARGO_MANIFEST_DIR").to_string();

        let image = File::open(format!("{}{}", base, path.into())).unwrap();

        let mut decoder = Decoder::new(image);
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().unwrap();

        let mut buf = vec![0; reader.output_buffer_size()];
        let frame = reader.next_frame(&mut buf).unwrap();
        let buf = &buf[..frame.buffer_size()];

        let info = reader.info();

        image_data.extend_from_slice(buf);

        Self {
            image_data,
            image_dimensions: [info.width, info.height],
        }
    }
}
