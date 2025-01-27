use std::io::Read;

use anyhow::{bail, ensure, Result};
use flate2::read::ZlibDecoder;

use crate::interlace::compute_pass_counts;
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
            "Invalid PNG file: incorrect signature.",
        );

        let chunks = self.parse_chunks()?;
        let mut chunks = chunks.into_iter();

        let Some(Chunk::ImageHeader(image_header)) = chunks.next() else {
            bail!("Expected image header chunk.");
        };

        ensure!(
            image_header.compression_method == 0,
            "Compression method should always be 0"
        );

        let mut chunks = chunks.peekable();

        // There may be multiple image data chunks. If so, they shall appear
        // consecutively with no intervening chunks. The compressed stream is then
        // the concatenation of the contents of all image data chunks.
        let mut compressed_stream = Vec::new();

        let mut gamma = 0;

        while let Some(chunk) = chunks.peek() {
            // todo, how would you collect palettes if ColorType::Palette?
            // todo, how do you collect ancillary chunks?
            if let &Chunk::Gamma(g) = chunk {
                gamma = g;
            }

            if let &Chunk::ImageData(sub_data) = chunk {
                compressed_stream.extend_from_slice(sub_data);
            }

            chunks.next();
        }

        let mut zlib_decoder = ZlibDecoder::new(&compressed_stream[..]);
        let mut input_buffer = Vec::new();
        zlib_decoder.read_to_end(&mut input_buffer)?;

        // filter
        ensure!(
            image_header.filter_method == 0,
            "Only filter method 0 is defined in the standard."
        );

        ensure!(!input_buffer.is_empty(), "Input buffer is empty.");

        let pixel_buffer = if image_header.interlace_method {
            self.adam7_deinterlace(&image_header, &input_buffer)?
        } else {
            self.non_interlaced(&image_header, &input_buffer)?
        };

        Ok(Png {
            width: image_header.width,
            height: image_header.height,
            gamma,
            color_type: image_header.color_type,
            pixel_buffer,
        })
    }

    fn adam7_deinterlace(
        &self,
        image_header: &ImageHeader,
        input_buffer: &'a [u8],
    ) -> Result<Vec<u8>> {
        let bytes_per_pixel = image_header.num_bytes_per_pixel();

        let mut pixel_buffer =
            vec![0u8; bytes_per_pixel * (image_header.height * image_header.width) as usize];

        let pass_counts = compute_pass_counts(image_header.width, image_header.height);
        let mut cursor = 0;

        for pass in pass_counts.into_iter() {
            let bytes_per_row = bytes_per_pixel * pass.width;

            for i in 0..pass.height {
                let mut row_start_idx = cursor + i * (1 + bytes_per_row);
                let filter_type = Filter::try_from(input_buffer[row_start_idx])?;
                row_start_idx += 1;

                let pixel_y = (pass.compute_y)(i);
                let row = &input_buffer[row_start_idx..row_start_idx + bytes_per_row];

                let _pixel_row_start = i * bytes_per_row;

                match filter_type {
                    Filter::None => {
                        for (j, pixel) in row.chunks_exact(bytes_per_pixel).enumerate() {
                            let pixel_x = (pass.compute_x)(j);

                            let index = (pixel_y * image_header.width as usize) + pixel_x;
                            pixel_buffer[index..index + bytes_per_pixel].copy_from_slice(pixel);
                        }
                    }
                    Filter::Sub => {
                        let mut new_row = row.to_vec();

                        for i in bytes_per_pixel..bytes_per_row {
                            let filtered = new_row[i].wrapping_add(new_row[i - bytes_per_pixel]);
                            new_row[i] = filtered;
                        }

                        for (j, pixel) in new_row.chunks_exact(bytes_per_pixel).enumerate() {
                            let pixel_x = (pass.compute_x)(j);

                            let index = (pixel_y * image_header.width as usize) + pixel_x;
                            pixel_buffer[index..index + bytes_per_pixel].copy_from_slice(pixel);
                        }
                    }
                    _ => todo!("What do other filters look like?"),
                }

                dbg!(filter_type);
            }

            cursor += (1 + bytes_per_row) * pass.height;
        }

        Ok(pixel_buffer)
    }

    fn non_interlaced(
        &self,
        image_header: &ImageHeader,
        input_buffer: &'a [u8],
    ) -> Result<Vec<u8>> {
        let mut pixel_buffer = vec![0_u8; input_buffer.len() - image_header.height as usize];

        let bytes_per_pixel = image_header.num_bytes_per_pixel();
        let bytes_per_row = bytes_per_pixel * image_header.width as usize;

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
                    pixel_buffer[pixel_row_start..pixel_row_start + bytes_per_pixel]
                        .copy_from_slice(&row[0..bytes_per_pixel]);

                    for j in bytes_per_pixel..bytes_per_row {
                        let filtered = row[j]
                            .wrapping_add(pixel_buffer[pixel_row_start + j - bytes_per_pixel]);

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
                        for j in 0..bytes_per_pixel {
                            pixel_buffer[pixel_row_start + j] = row[j].wrapping_add(
                                (pixel_buffer[pixel_row_start - bytes_per_row + j] as u16 / 2)
                                    as u8,
                            )
                        }
                    } else {
                        pixel_buffer[pixel_row_start..pixel_row_start + bytes_per_pixel]
                            .copy_from_slice(&row[0..bytes_per_pixel]);
                    }

                    for j in bytes_per_pixel..bytes_per_row {
                        let mut a = pixel_buffer[pixel_row_start + j - bytes_per_pixel] as u16;

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
                        let left = if j < bytes_per_pixel {
                            0
                        } else {
                            pixel_buffer[pixel_row_start + j - bytes_per_pixel]
                        };

                        let up_left = if j < bytes_per_pixel || pixel_row_start < bytes_per_row {
                            0
                        } else {
                            pixel_buffer[pixel_row_start - bytes_per_row + j - bytes_per_pixel]
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

        Ok(pixel_buffer)
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

        loop {
            let length = self.read_u32()? as usize;

            {
                ensure!(
                    self.cursor + 4 + length < self.data.len(),
                    "EOF: Failed to read CRC value."
                );

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
                        compression_method: self.read_u8()?,
                        filter_method: self.read_u8()?,
                        interlace_method: self.read_u8()? == 1,
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
                b"gAMA" => Chunk::Gamma(self.read_u32()?),
                b"sRGB" => todo!("Parse srgb chunks"),
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
    use super::*;
    use crate::test_file_parser::parse_test_file;
    use anyhow::anyhow;
    use pretty_assertions::assert_eq;

    #[allow(dead_code)]
    fn generate_blob(path: &str) -> Result<()> {
        let content = std::fs::read(format!("{}.png", path))?;
        let png = Decoder::new(&content).decode()?;

        png.write_to_binary_blob(path)?;

        Ok(())
    }

    fn compare_png(image_title: &str) -> Result<()> {
        let expected_png = Png::read_from_binary_blob(&format!("./tests/{}", image_title).into())
            .map_err(|err| anyhow!("Try regenerating the blob: {:?}", err))?;

        let path = format!("./tests/{}.png", image_title);

        let content = std::fs::read(&path)?;
        let generated_png = Decoder::new(&content).decode()?;

        if expected_png != generated_png {
            assert_eq!(
                expected_png,
                generated_png,
                "Failed test: {:?}",
                parse_test_file(&path.into())?.test_desc
            );
        }

        Ok(())
    }

    // A note about the following test cases, these images were hand checked. This way, binary blobs can be generated with confidence (or hubris).

    #[test]
    fn test_basic_grayscale_8bit() -> Result<()> {
        // generate_blob("./tests/basn0g08")?;
        compare_png("basn0g08")?;
        Ok(())
    }

    #[test]
    fn test_basic_grayscale_alpha_8bit() -> Result<()> {
        // generate_blob("./tests/basn4a08")?;
        compare_png("basn4a08")?;
        Ok(())
    }

    #[test]
    fn test_basic_rgb_alpha_8bit() -> Result<()> {
        // generate_blob("./tests/basn6a08")?;
        compare_png("basn6a08")?;
        Ok(())
    }

    #[test]
    fn test_filter_0() -> Result<()> {
        // generate_blob("./tests/f00n2c08")?;
        compare_png("f00n2c08")?;

        // generate_blob("./tests/f00n0g08")?;
        compare_png("f00n0g08")?;

        Ok(())
    }

    #[test]
    fn test_filter_1() -> Result<()> {
        // generate_blob("./tests/f01n2c08")?;
        compare_png("f01n2c08")?;

        // generate_blob("./tests/f01n0g08")?;
        compare_png("f01n0g08")?;
        Ok(())
    }

    #[test]
    fn test_filter_2() -> Result<()> {
        // generate_blob("./tests/f02n2c08")?;
        compare_png("f02n2c08")?;

        // generate_blob("./tests/f02n0g08")?;
        compare_png("f02n0g08")?;
        Ok(())
    }

    #[test]
    fn test_filter_3() -> Result<()> {
        // generate_blob("./tests/f03n2c08")?;
        compare_png("f03n2c08")?;

        // generate_blob("./tests/f03n0g08")?;
        compare_png("f03n0g08")?;
        Ok(())
    }

    #[test]
    fn test_filter_4() -> Result<()> {
        // generate_blob("./tests/f04n2c08")?;
        compare_png("f04n2c08")?;

        // generate_blob("./tests/f04n0g08")?;
        compare_png("f04n0g08")?;
        Ok(())
    }
}
