use anyhow::{ensure, Result};

use crate::grammar::ZLib;

#[derive(Debug)]
pub struct DeflateDecompressor {
    cursor: usize,
    data: Vec<u8>,
}

impl DeflateDecompressor {
    pub fn new(data: Vec<u8>) -> Self {
        Self { cursor: 0, data }
    }

    pub fn decompress(&mut self) -> Result<()> {
        let zlib = self.parse_zlib()?;

        ensure!(
            zlib.compression_method() == 8,
            "Expected compression method 8. Got: {}.",
            zlib.compression_method()
        );

        ensure!(
            zlib.compression_info() == 7,
            "Expected compression method 7. Got: {}.",
            zlib.compression_info()
        );

        let _compressed = &self.data[self.cursor..];

        Ok(())
    }

    fn parse_zlib(&mut self) -> Result<ZLib> {
        Ok(ZLib {
            compression_method_flags: self.read_u8()?,
            additional_flags: self.read_u8()?,
            check_value: self.read_u32_trailing()?,
        })
    }

    fn read_u8(&mut self) -> Result<u8> {
        let end = self.data.len();

        ensure!(
            self.cursor < end,
            "Unexpected EOF. At {}, buffer size: {}.",
            self.cursor,
            end
        );

        let b = self.data[self.cursor];
        self.cursor += 1;

        Ok(b)
    }

    fn read_u32_trailing(&mut self) -> Result<u32> {
        let span = self.data.len() - self.cursor;

        ensure!(span >= 4, "Unexpected EOF.");
        let n = u32::from_be_bytes(self.data[self.data.len() - 4..].try_into()?);

        self.data.truncate(span);

        Ok(n)
    }
}
