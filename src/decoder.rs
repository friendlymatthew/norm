use std::{collections::BTreeMap, io::Read};

use anyhow::{bail, ensure, Result};
use flate2::read::ZlibDecoder;

use crate::{crc32::compute_crc, Chunk, ColorType, Filter, ImageHeader, Png};

#[derive(Debug)]
pub struct Decoder<'a> {
    cursor: usize,
    data: &'a [u8],
}

impl<'a> Decoder<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { cursor: 0, data }
    }

    pub fn decode(&mut self) -> Result<Png> {
        ensure!(
            self.read_slice(8)? == b"\x89PNG\r\n\x1A\n",
            "Expected signature.",
        );

        let chunks = self.parse_chunks()?;
        let mut chunks = chunks.into_iter();

        let Some(Chunk::ImageHeader(image_header)) = chunks.next() else {
            bail!("Expected image header chunk.");
        };

        let mut chunks = chunks.peekable();

        // There may be multiple image data chunks. If so, they shall appear
        // consecutively with no intervening chunks. The compressed stream is then
        // the concatenation of the contents of all image data chunks.
        let mut compressed_stream = Vec::new();

        while let Some(chunk) = chunks.peek() {
            // todo, how would you collect palettes if ColorType::Palette?
            // todo, how do you collect ancillary chunks?

            if let &Chunk::ImageData(sub_data) = chunk {
                compressed_stream.extend_from_slice(sub_data);
            }

            chunks.next();
        }

        // todo!, what if ancillary chunks appear after the image data chunks?

        let mut zlib_decoder = ZlibDecoder::new(&compressed_stream[..]);
        let mut input_buffer = Vec::new();
        zlib_decoder.read_to_end(&mut input_buffer)?;

        // filter
        ensure!(
            image_header.filter_method == 0,
            "Only filter method 0 is defined in the standard."
        );

        ensure!(!input_buffer.is_empty(), "Input buffer is empty.");

        let mut pixel_buffer = vec![0_u8; input_buffer.len() - image_header.height as usize];

        let num_channels = image_header.color_type.num_channels() as usize;
        let bytes_per_row = image_header.num_bytes_per_pixel() * image_header.width as usize;

        for i in 0..image_header.height as usize {
            let mut row_start_idx = i * (1 + bytes_per_row);
            let filter_type = Filter::try_from(input_buffer[row_start_idx])?;
            row_start_idx += 1;
            let row = &input_buffer[row_start_idx..row_start_idx + bytes_per_row];

            let pixel_row_start = i * bytes_per_row;

            match filter_type {
                Filter::None => {
                    // the best filter.
                    pixel_buffer[pixel_row_start..pixel_row_start + bytes_per_row]
                        .copy_from_slice(row);
                }
                Filter::Sub => {
                    pixel_buffer[pixel_row_start..pixel_row_start + num_channels]
                        .copy_from_slice(&row[0..num_channels]);

                    for j in num_channels..bytes_per_row {
                        let filtered =
                            row[j].wrapping_add(pixel_buffer[pixel_row_start + j - num_channels]);

                        pixel_buffer[pixel_row_start + j] = filtered;
                    }
                }
                Filter::Up => {
                    if pixel_row_start < bytes_per_row {
                        pixel_buffer[pixel_row_start..pixel_row_start + bytes_per_row]
                            .copy_from_slice(row);

                        continue;
                    }

                    for j in 0..bytes_per_row {
                        let filtered =
                            row[j].wrapping_add(pixel_buffer[pixel_row_start - bytes_per_row + j]);

                        pixel_buffer[pixel_row_start + j] = filtered;
                    }
                }
                Filter::Average => {
                    let has_prev_row = pixel_row_start >= bytes_per_row;

                    if has_prev_row {
                        for j in 0..num_channels {
                            pixel_buffer[pixel_row_start + j] = row[j].wrapping_add(
                                (pixel_buffer[pixel_row_start - bytes_per_row + j] as u16 / 2)
                                    as u8,
                            )
                        }
                    } else {
                        pixel_buffer[pixel_row_start..pixel_row_start + num_channels]
                            .copy_from_slice(&row[0..num_channels]);
                    }

                    for j in num_channels..bytes_per_row {
                        let mut a = pixel_buffer[pixel_row_start + j - num_channels] as u16;

                        if has_prev_row {
                            a += pixel_buffer[pixel_row_start - bytes_per_row + j] as u16;
                        }

                        a /= 2;

                        let filtered = row[j].wrapping_add(a as u8);
                        pixel_buffer[pixel_row_start + j] = filtered;
                    }
                }
                Filter::Paeth => {
                    for j in 0..row.len() {
                        let left = if j < num_channels {
                            0
                        } else {
                            pixel_buffer[pixel_row_start + j - num_channels]
                        };

                        let up_left = if j < num_channels || pixel_row_start < bytes_per_row {
                            0
                        } else {
                            pixel_buffer[pixel_row_start - bytes_per_row + j - num_channels]
                        };

                        let filtered = row[j].wrapping_add(self.paeth(
                            left,
                            if pixel_row_start < bytes_per_row {
                                0
                            } else {
                                pixel_buffer[pixel_row_start - bytes_per_row + j]
                            },
                            up_left,
                        ));

                        pixel_buffer[pixel_row_start + j] = filtered;
                    }
                }
            }
        }

        ensure!(pixel_buffer.len() == input_buffer.len() - image_header.height as usize);

        Ok(Png {
            width: image_header.width,
            height: image_header.height,
            color_type: image_header.color_type,
            pixel_buffer,
        })
    }

    const fn paeth(&self, left: u8, up: u8, up_left: u8) -> u8 {
        let a = left as i16;
        let b = up as i16;
        let c = up_left as i16;

        let p = a + b - c;

        let pa = (p - a).abs();
        let pb = (p - b).abs();
        let pc = (p - c).abs();

        if pa <= pb && pa <= pc {
            left
        } else if pb <= pc {
            up
        } else {
            up_left
        }
    }

    fn validate_crc(&self, chunk_type: &'a [u8], chunk_data: &'a [u8], expected_crc: u32) -> bool {
        expected_crc == compute_crc(chunk_type, chunk_data)
    }

    fn parse_chunks(&mut self) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let mut text_map = BTreeMap::new();

        loop {
            let length = self.read_u32()? as usize;

            {
                let expected_crc = u32::from_be_bytes(
                    self.data[self.cursor + 4 + length..self.cursor + 4 + length + 4].try_into()?,
                );

                ensure!(self.validate_crc(
                    &self.data[self.cursor..self.cursor + 4],
                    &self.data[self.cursor + 4..self.cursor + 4 + length],
                    expected_crc
                ));
            }

            let chunk = match self.read_slice(4)? {
                b"IHDR" => {
                    ensure!(chunks.is_empty(), "ImageHeader chunk must appear first.");

                    Chunk::ImageHeader(ImageHeader {
                        width: self.read_u32()?,
                        height: self.read_u32()?,
                        bit_depth: self.read_u8()?,
                        color_type: self.read_u8()?.try_into()?,
                        _compression_method: self.read_u8()?,
                        filter_method: self.read_u8()?,
                        _interlace_method: self.read_u8()? == 1,
                    })
                }
                b"PLTE" => {
                    ensure!(length % 3 == 0, "Chunk length not divisible by 3.");
                    ensure!(
                        !chunks.is_empty(),
                        "Empty chunks. Expected ImageHeader chunk."
                    );

                    let Chunk::ImageHeader(image_header) = &chunks[0] else {
                        bail!("Expected ImageHeader chunk.");
                    };

                    let color_type = image_header.color_type;

                    ensure!(
                        !matches!(color_type, ColorType::Grayscale)
                            && !matches!(color_type, ColorType::GrayscaleAlpha)
                    );

                    if color_type != ColorType::Palette {
                        self.skip_crc()?;
                        continue;
                    }

                    let entries = self.read_slice(length)?.chunks_exact(3);
                    Chunk::Palette(entries)
                }
                b"IDAT" => Chunk::ImageData(self.read_slice(length)?),
                b"IEND" => break,
                b"tEXt" => {
                    let mut split = self.read_slice(length)?.split(|x| *x == 0x00);

                    if let (Some(keyword), Some(text_string)) = (split.next(), split.next()) {
                        text_map
                            .entry(keyword)
                            .and_modify(|e| *e = text_string)
                            .or_insert_with(|| text_string);
                    } else {
                        bail!("Invalid text chunk");
                    };

                    self.skip_crc()?;
                    continue;
                }
                b"gAMA" => Chunk::Gamma(self.read_u32()?),
                // b"zTXt" => {
                //     let mut split = self.read_slice(length)?.split(|x| *x == 0x00);

                //     if let (Some(keyword), Some(compressed_text)) = (split.next(), split.next()) {
                //         let _compression_method = compressed_text[0];

                //         let mut zlib_decoder = ZlibDecoder::new(&compressed_text[1..]);
                //         let mut text_string = Vec::new();
                //         zlib_decoder.read_to_end(&mut text_string)?;

                //         text_map.entry(keyword).and_modify(|e| e.push(text_string));
                //     } else {
                //         bail!("Invalid text chunk");
                //     };

                //     let _crc = self.read_u32()?;
                //     continue;
                // }
                _foreign => {
                    // todo! how would ancillary chunks be parsed?
                    self.cursor += length;
                    self.skip_crc()?;
                    continue;
                }
            };

            self.skip_crc()?;

            chunks.push(chunk);
        }

        if !text_map.is_empty() {
            chunks.push(Chunk::TextData(text_map));
        }

        Ok(chunks)
    }

    fn skip_crc(&mut self) -> Result<()> {
        self.eof(4)?;
        self.cursor += 4;

        Ok(())
    }

    fn eof(&self, len: usize) -> Result<()> {
        let end = self.data.len();

        ensure!(
            self.cursor + len.saturating_sub(1) < self.data.len(),
            "Unexpected EOF. At {}, seek by {}, buffer size: {}.",
            self.cursor,
            len,
            end
        );

        Ok(())
    }

    fn read_u8(&mut self) -> Result<u8> {
        self.eof(0)?;

        let b = self.data[self.cursor];
        self.cursor += 1;

        Ok(b)
    }

    fn read_u32(&mut self) -> Result<u32> {
        self.eof(4)?;

        let slice = &self.data[self.cursor..self.cursor + 4];
        let n = u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]);

        self.cursor += 4;

        Ok(n)
    }

    fn read_slice(&mut self, len: usize) -> Result<&'a [u8]> {
        self.eof(len)?;

        let slice = &self.data[self.cursor..self.cursor + len];
        self.cursor += len;

        Ok(slice)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use crate::test_file_parser::TestFileParser;

    use super::*;

    const TEST_FILE_PARSER: TestFileParser = TestFileParser;

    #[allow(dead_code)]
    fn generate_blob(path: &str) -> Result<()> {
        let content = std::fs::read(format!("{}.png", path))?;
        let png = Decoder::new(&content).decode()?;

        png.write_to_binary_blob(path)?;

        Ok(())
    }

    fn compare_png(image_title: &str) -> Result<()> {
        let expected_png = Png::read_from_binary_blob(&format!("./tests/{}", image_title))
            .map_err(|err| anyhow!("Try regenerating the blob: {:?}", err))?;

        let content = std::fs::read(format!("./tests/{}.png", image_title))?;
        let generated_png = Decoder::new(&content).decode()?;

        if expected_png != generated_png {
            let tf = TEST_FILE_PARSER.parse(format!("./tests/{}.png", image_title).into())?;
            assert_eq!(expected_png, generated_png, "Failed test: {:?}", tf);
        }

        Ok(())
    }

    #[test]
    fn test_filter_0() -> Result<()> {
        // generate_blob("./tests/f00n2c08")?;
        compare_png("f00n2c08")?;

        Ok(())
    }

    #[test]
    fn test_filter_1() -> Result<()> {
        // generate_blob("./tests/f01n2c08")?;
        compare_png("f01n2c08")?;

        Ok(())
    }

    #[test]
    fn test_filter_2() -> Result<()> {
        // generate_blob("./tests/f02n2c08")?;
        compare_png("f02n2c08")?;

        Ok(())
    }

    #[test]
    fn test_filter_3() -> Result<()> {
        // generate_blob("./tests/f03n2c08")?;
        compare_png("f03n2c08")?;

        Ok(())
    }

    #[test]
    fn test_filter_4() -> Result<()> {
        // generate_blob("./tests/f04n2c08")?;
        compare_png("f04n2c08")?;

        Ok(())
    }
}
