use getset::{CopyGetters, Getters};
use png::{Decoder, Transformations};
use std::fs::File;
use std::io::BufReader;

#[derive(CopyGetters, Getters, Clone)]
pub struct Texture {
    #[getset(get = "pub with_prefix")]
    image_data: Vec<u8>,
    #[getset(get = "pub with_prefix")]
    image_dimensions: [u32; 2],
}

impl Texture {
    pub fn new(path: impl AsRef<str>) -> Self {
        let full_path = format!("{}{}", env!("CARGO_MANIFEST_DIR"), path.as_ref());
        let file = File::open(full_path).unwrap();

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
}