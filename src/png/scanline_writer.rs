use crate::png::grammar::{Filter, ImageHeader};
use anyhow::Result;
use std::io::Write;

const fn paeth_predict(orig_a: u8, orig_b: u8, orig_c: u8) -> u8 {
    let (a, b, c) = (orig_a as i16, orig_b as i16, orig_c as i16);

    let p = a + b + c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();

    if pa <= pb && pa <= pc {
        orig_a
    } else if pb <= pc {
        orig_b
    } else {
        orig_c
    }
}

/// Computes the output scanline using all five filters, and select the filter that gives the
/// smallest sum of absolute values of outputs.
fn test_filters(prev_chunk: &[u8], chunk: &[u8], num_bytes_per_pixel: usize) -> (Filter, Vec<u8>) {
    let mut sub_scanline = Vec::with_capacity(chunk.len());
    let mut up_scanline = Vec::with_capacity(chunk.len());
    let mut avg_scanline = Vec::with_capacity(chunk.len());
    let mut paeth_scanline = Vec::with_capacity(chunk.len());

    chunk.iter().enumerate().for_each(|(i, &orig)| {
        let a = if i < num_bytes_per_pixel {
            0
        } else {
            chunk[i - num_bytes_per_pixel]
        };

        let b = prev_chunk[i];

        let c = if i < num_bytes_per_pixel {
            0
        } else {
            prev_chunk[i - num_bytes_per_pixel]
        };

        sub_scanline.push(orig.wrapping_sub(a));
        up_scanline.push(orig.wrapping_sub(b));
        avg_scanline.push(orig.wrapping_sub(((a as u16 + b as u16) / 2) as u8));
        paeth_scanline.push(orig.wrapping_sub(paeth_predict(a, b, c)));
    });

    vec![
        (Filter::None, chunk.to_vec()),
        (Filter::Sub, sub_scanline),
        (Filter::Up, up_scanline),
        (Filter::Average, avg_scanline),
        (Filter::Paeth, paeth_scanline),
    ]
    .into_iter()
    .min_by_key(|(_, scanline)| scanline.iter().map(|&b| b as u32).sum::<u32>())
    .unwrap()
}

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
        let num_bytes_per_pixel = self.image_header.num_bytes_per_pixel();

        assert_eq!(
            self.image_header.width as usize
                * self.image_header.height as usize
                * num_bytes_per_pixel,
            pixel_buffer.len()
        );

        let scanline_bytes = self.image_header.width as usize * num_bytes_per_pixel;

        let mut prev_chunk: &[u8] = &vec![0u8; scanline_bytes];

        for chunk in pixel_buffer.chunks_exact(scanline_bytes) {
            let (filter, scanline) = test_filters(prev_chunk, chunk, num_bytes_per_pixel);

            self.writer.write_all(&[filter as u8])?;
            self.writer.write_all(&scanline)?;

            prev_chunk = chunk;
        }

        Ok(())
    }

    pub fn finish(self) -> W {
        self.writer
    }
}
