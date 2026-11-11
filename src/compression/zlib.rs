use crate::{compression::deflate, impl_read_for_datatype_be, impl_read_slice};
use anyhow::{ensure, Result};

#[derive(Debug)]
pub struct ZlibDecoder<'a> {
    cursor: usize,
    data: &'a [u8],
}

impl<'a> ZlibDecoder<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0 }
    }

    pub fn decode(&mut self) -> Result<Vec<u8>> {
        let cmf = self.read_u8()?;
        let flg = self.read_u8()?;

        ensure!(cmf & 0x0f == 8, "compression method must be deflate");
        ensure!(cmf >> 4 <= 7, "invalid window size");
        ensure!(
            u16::from_be_bytes([cmf, flg]) % 31 == 0,
            "invalid zlib header"
        );
        ensure!(flg & 0x20 == 0, "preset dictionary unsupported");

        let remaining = self.data.len() - self.cursor;
        ensure!(remaining >= 4, "missing Adler-32 checksum");

        let compressed = self.read_slice(remaining - 4)?;
        let _expected_adler32 = self.read_u32()?;

        deflate::decode(compressed)
    }

    impl_read_for_datatype_be!(read_u8, u8);
    impl_read_for_datatype_be!(read_u32, u32);

    impl_read_slice!();
}
