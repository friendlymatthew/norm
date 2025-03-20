use crate::png::chunk::{IDATChunk, IENDChunk, IHDRChunk, PngChunk};
use crate::png::grammar::Png;
use anyhow::Result;
use std::io::Write;

pub struct PngEncoder<W: Write> {
    writer: W,
}

impl<W: Write> PngEncoder<W> {
    pub const fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn encode(&mut self, png: Png) -> Result<()> {
        self.writer.write_all(b"\x89PNG\r\n\x1A\n")?;

        let Png {
            image_header,
            pixel_buffer,
            ..
        } = png;

        let image_header_chunk = IHDRChunk { image_header };
        image_header_chunk.write(&mut self.writer)?;

        // let palette_chunk = PLTEChunk;
        // palette_chunk.write(&mut self.writer)?;

        let image_data_chunk = IDATChunk { data: pixel_buffer };
        image_data_chunk.write(&mut self.writer)?;

        let image_end = IENDChunk;
        image_end.write(&mut self.writer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::png::PngDecoder;
    use std::fs::File;

    #[test]
    fn test_encode() -> Result<()> {
        let data = std::fs::read("./tests/obama.png")?;
        let png = PngDecoder::new(&data).decode()?;

        let file = File::create("./tests/obama_encoded.png")?;
        let mut encoder = PngEncoder::new(file);

        encoder.encode(png)?;

        Ok(())
    }
}
