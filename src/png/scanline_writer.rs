use crate::png::grammar::{Filter, ImageHeader};
use anyhow::Result;
use std::io::Write;

#[derive(Debug)]
pub struct ScanlineWriter<'a, W: Write> {
    image_header: &'a ImageHeader,
    writer: W,
}

impl<'a, W: Write> ScanlineWriter<'a, W> {
    pub const fn new(writer: W, image_header: &'a ImageHeader) -> Self {
        Self {
            writer,
            image_header,
        }
    }

    pub fn write(&mut self, pixel_buffer: &'a [u8]) -> Result<()> {
        assert_eq!(
            self.image_header.width as usize
                * self.image_header.height as usize
                * self.image_header.num_bytes_per_pixel(),
            pixel_buffer.len()
        );

        for chunk in pixel_buffer.chunks_exact(
            self.image_header.width as usize * self.image_header.num_bytes_per_pixel(),
        ) {
            self.writer.write_all(&[Filter::None as u8])?;
            self.writer.write_all(chunk)?;
        }

        Ok(())
    }

    pub fn finish(self) -> W {
        self.writer
    }
}
