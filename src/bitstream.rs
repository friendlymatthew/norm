use anyhow::{ensure, Result};

#[derive(Debug)]
pub struct BitstreamReader<'a> {
    // in bits
    cursor: usize,
    data: &'a [u8],
}

impl<'a> BitstreamReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { cursor: 0, data }
    }

    fn divmod_8(&self) -> (usize, usize) {
        (self.cursor / 8, self.cursor % 8)
    }

    fn read_bit_unchecked(&mut self) -> u8 {
        let (byte_idx, bit_idx) = self.divmod_8();
        let byte = self.data[byte_idx];

        self.cursor += 1;

        (byte >> bit_idx) & 0b1
    }

    pub fn read_bit(&mut self) -> Result<u8> {
        ensure!(self.cursor < self.data.len() * 8, "Unexpected EOF.");

        Ok(self.read_bit_unchecked())
    }

    pub fn read_nbits(&mut self, bit_len: usize) -> Result<usize> {
        ensure!(
            self.cursor + bit_len <= self.data.len() * 8,
            "Unexpected EOF."
        );

        let mut out = 0_usize;

        for i in 0..bit_len {
            let b = self.read_bit_unchecked();
            out |= (b as usize) << i;
        }

        Ok(out)
    }

    fn byte_aligned(&self) -> Result<usize> {
        let (byte_idx, bit_len) = self.divmod_8();
        ensure!(
            bit_len == 0,
            "Cursor must be byte aligned. Currently partially aligned with {} bits read at {}",
            bit_len,
            byte_idx,
        );

        Ok(byte_idx)
    }

    /// NOTE: this is under the assumption the cursor is byte aligned.
    pub fn read_u16(&mut self) -> Result<u16> {
        let byte_idx = self.byte_aligned()?;
        let n = u16::from_be_bytes(self.data[byte_idx..byte_idx + 2].try_into()?);
        self.cursor += 16;

        Ok(n)
    }

    /// NOTE: this is under the assumption the cursor is byte aligned.
    pub fn read_slice(&mut self, byte_len: usize) -> Result<&'a [u8]> {
        let byte_idx = self.byte_aligned()?;

        ensure!(byte_len > 0, "Requested length must not be nonzero.");
        ensure!(byte_idx + byte_len - 1 < self.data.len(), "Unexpected EOF.");

        let slice = &self.data[byte_idx..byte_idx + byte_len];
        self.cursor += byte_len * 8;

        Ok(slice)
    }

    pub fn next_byte(&mut self) -> Result<()> {
        ensure!(self.cursor < self.data.len() * 8, "Unexpected EOF.");

        let (_, bit_len) = self.divmod_8();

        dbg!(bit_len);

        if bit_len != 0 {
            self.cursor += 8 - bit_len;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_nbits() -> Result<()> {
        let data = [0b1111_1010];
        let mut bitstream = BitstreamReader::new(&data);

        assert_eq!(0, bitstream.read_bit()?);

        assert_eq!(0b01, bitstream.read_nbits(2)?);
        assert_eq!(0b11111, bitstream.read_nbits(5)?);

        assert!(bitstream.read_nbits(1).is_err());

        Ok(())
    }

    #[test]
    fn test_read_bit() -> Result<()> {
        let data = [0b0101_1010];
        let mut bitstream = BitstreamReader::new(&data);

        assert_eq!(0b101_1010, bitstream.read_nbits(7)?);
        assert_eq!(0b0, bitstream.read_bit()?);
        assert!(bitstream.read_bit().is_err());

        Ok(())
    }

    #[test]
    fn test_next_byte() -> Result<()> {
        let data = [0, 0b1111_1010, 0b0101_0101];
        let mut bitstream = BitstreamReader::new(&data);

        assert_eq!(0, bitstream.read_nbits(6)?);
        bitstream.next_byte()?;

        assert_eq!(0b010, bitstream.read_nbits(3)?);
        assert_eq!(0b1_1111, bitstream.read_nbits(5)?);
        bitstream.next_byte()?;

        assert_eq!(0b0101_0101, bitstream.read_nbits(8)?);

        Ok(())
    }
}
