use crate::{
    impl_read_for_datatype, impl_read_slice,
    jpeg::grammar::{
        Component, EncodingProcess, HuffmanTable, Jpeg, Marker, Precision, QuantizationTable,
        StartOfFrame, StartOfScan, JFIF,
    },
};

use anyhow::{anyhow, ensure, Result};
use std::ops::{Range, RangeInclusive};

#[derive(Debug)]
pub struct JpegDecoder<'a> {
    cursor: usize,
    data: &'a [u8],
}

impl<'a> JpegDecoder<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { cursor: 0, data }
    }

    pub fn decode(&mut self) -> Result<Jpeg> {
        let _jfif = self.parse_jfif()?;

        dbg!(_jfif);

        todo!();
    }

    fn parse_jfif(&mut self) -> Result<JFIF> {
        ensure!(self.read_marker()? == 0xFFD8);

        let mut quantization_tables = Vec::with_capacity(4);
        let mut huffman_tables = Vec::new();
        let mut start_of_frame = None;
        let mut start_of_scan = None;
        let mut image_data = None;

        loop {
            match self.read_marker()? {
                0xFFE0 => {
                    self.parse_application_header()?;
                }
                0xFFDB => {
                    quantization_tables.push(self.parse_quantization_table()?);
                }
                0xFFC4 => {
                    huffman_tables.push(self.parse_huffman_table()?);
                }
                0xFFDA => {
                    ensure!(start_of_scan.is_none() && image_data.is_none());
                    start_of_scan = Some(self.parse_start_of_scan()?);
                    image_data = Some(self.parse_image_data()?);

                    break;
                }
                start_of_frame_marker if (start_of_frame_marker as u8 & 0xF0) == 0xC0 => {
                    ensure!(start_of_frame.is_none());
                    start_of_frame = Some(self.parse_start_of_frame(start_of_frame_marker as u8)?);
                }
                foreign => unimplemented!("{:X}", foreign),
            };
        }

        ensure!(self.read_marker()? == 0xFFD9);

        Ok(JFIF {
            quantization_tables,
            huffman_tables: {
                ensure!(huffman_tables.len() == 4);
                huffman_tables
            },
            start_of_frame: start_of_frame.ok_or_else(|| anyhow!("expected start of frame"))?,
            start_of_scan: start_of_scan.ok_or_else(|| anyhow!("expected start of scan"))?,
            image_data: image_data.ok_or_else(|| anyhow!("expected image data"))?,
        })
    }

    fn parse_application_header(&mut self) -> Result<()> {
        let offset = self.cursor;
        let length = self.read_u16()?;

        ensure!(self.read_slice(5)? == b"JFIF\0");

        self.cursor = offset + length as usize;

        Ok(())
    }

    fn parse_quantization_table(&mut self) -> Result<QuantizationTable> {
        let offset = self.cursor;
        let length = self.read_u16()? as usize;

        let flag = self.read_u8()?;

        let precision = Precision::from((flag >> 4) == 1);

        ensure!(
            self.cursor + (precision as usize * QuantizationTable::NUM_ELEMENTS) == offset + length
        );

        let table_elements = match precision {
            Precision::Eight => {
                self.read_fixed_array::<64, _>(|this| this.read_u8().map(|b| b as u16))?
            }
            Precision::Sixteen => self.read_fixed_array::<64, _>(Self::read_u16)?,
        };

        Ok(QuantizationTable {
            flag,
            table_elements,
        })
    }

    fn parse_start_of_frame(&mut self, start_of_frame: u8) -> Result<StartOfFrame> {
        let encoding_process = EncodingProcess::try_from(start_of_frame & 0b1111)?;

        let offset = self.cursor;
        let length = self.read_u16()?;

        let start_of_frame = StartOfFrame {
            encoding_process,
            sample_precision: self.read_u8()?,
            lines: self.read_u16()?,
            samples_per_line: self.read_u16()?,
            components: {
                let number_of_image_components = self.read_u8()?;
                self.read_vec(number_of_image_components as usize, Self::parse_component)?
            },
        };

        ensure!(self.cursor == offset + length as usize);

        Ok(start_of_frame)
    }

    fn parse_component(&mut self) -> Result<Component> {
        Ok(Component {
            identifier: self.read_u8()?,
            sampling_factor: self.read_u8()?,
            quantization_table_destination_selector: self.read_u8()?,
        })
    }

    fn parse_huffman_table(&mut self) -> Result<HuffmanTable> {
        let offset = self.cursor;
        let length = self.read_u16()? as usize;

        let flag = self.read_u8()?;

        let huffman_table = HuffmanTable {
            flag,
            code_lengths: {
                let code_lengths = Range {
                    start: self.cursor,
                    end: self.cursor + 16,
                };

                self.cursor += 16;

                code_lengths
            },
            symbols: Range {
                start: self.cursor,
                end: offset + length,
            },
        };

        self.cursor = offset + length;

        Ok(huffman_table)
    }

    fn parse_start_of_scan(&mut self) -> Result<StartOfScan> {
        let offset = self.cursor;
        let length = self.read_u16()?;

        let number_of_image_components = self.read_u8()?;
        let components = self.read_vec(number_of_image_components as usize, |this| {
            Ok((this.read_u8()?, this.read_u8()?))
        })?;

        let start_of_scan = StartOfScan {
            components,
            spectral_select: RangeInclusive::new(self.read_u8()?, self.read_u8()?),
            approximation: self.read_u8()?,
        };

        ensure!(self.cursor == offset + length as usize);

        Ok(start_of_scan)
    }

    fn parse_image_data(&mut self) -> Result<Range<usize>> {
        let range = Range {
            start: self.cursor,
            end: {
                while self.peek_slice(2)? != [0xFF, 0xD9] {
                    self.cursor += 1;
                }

                self.cursor
            },
        };

        Ok(range)
    }

    impl_read_for_datatype!(read_u8, u8);
    impl_read_for_datatype!(read_u16, u16);
    impl_read_for_datatype!(read_marker, Marker);
    impl_read_slice!();
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_decode_taxi_zone_map_manhattan() {
//         let data = std::fs::read("./tests/taxi_zone_map_manhattan.jpg").unwrap();

//         let _ = JpegDecoder::new(&data).decode().unwrap();
//     }
// }
