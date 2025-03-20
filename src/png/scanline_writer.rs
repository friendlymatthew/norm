use crate::png::grammar::{Filter, ImageHeader};
use anyhow::Result;
use std::io::Write;

#[derive(Debug)]
pub struct ScanlineWriter<'a> {
    image_header: &'a ImageHeader,
}

impl<'a> ScanlineWriter<'a> {
    pub const fn new(image_header: &'a ImageHeader) -> Self {
        Self { image_header }
    }

    pub fn write<W: Write>(&self, writer: &mut W, pixel_buffer: &'a [u8]) -> Result<()> {
        assert_eq!(
            self.image_header.width as usize
                * self.image_header.height as usize
                * self.image_header.num_bytes_per_pixel(),
            pixel_buffer.len()
        );

        for chunk in pixel_buffer.chunks_exact(
            self.image_header.width as usize * self.image_header.num_bytes_per_pixel(),
        ) {
            writer.write_all(&[Filter::None as u8])?;
            writer.write_all(chunk)?;
        }

        Ok(())
    }
}
