use anyhow::{anyhow, ensure, Result};

#[derive(Debug)]
pub struct BitReader<'a> {
    data: &'a [u8],

    byte_pos: usize,
    buf: u64,
    count: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,

            byte_pos: 0,
            buf: 0,
            count: 0,
        }
    }

    pub const fn is_byte_aligned(&self) -> bool {
        self.count.is_multiple_of(8)
    }

    pub fn align_to_byte(&mut self) {
        let leftover = self.count % 8;
        self.buf >>= leftover;
        self.count -= leftover;
    }

    pub fn read_bits(&mut self, n: usize) -> Result<usize> {
        debug_assert!(
            n <= 16,
            "deflate should never read more than 16 bits at a time"
        );

        if self.count < n {
            self.refill();
            ensure!(self.count >= n, "unexpected EOF");
        }

        let out = self.buf & ((1 << n) - 1);
        self.buf >>= n;
        self.count -= n;

        Ok(out as usize)
    }

    pub fn read_bit(&mut self) -> Result<usize> {
        self.read_bits(1)
    }

    pub fn read_array<const N: usize>(&mut self) -> Result<&'a [u8; N]> {
        let out = self.read_bytes(N)?;
        out.try_into().map_err(|e| anyhow!("{e:?}"))
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        debug_assert!(self.is_byte_aligned(), "not byte aligned");

        let start = self.byte_pos - self.count / 8;

        let end = start
            .checked_add(len)
            .ok_or_else(|| anyhow!("unexpected eof"))?;

        let out = self
            .data
            .get(start..end)
            .ok_or_else(|| anyhow!("unexpected eof"))?;

        self.byte_pos = end;
        self.buf = 0;
        self.count = 0;

        Ok(out)
    }

    fn refill(&mut self) {
        while self.count <= 56 && self.byte_pos < self.data.len() {
            // safety: the loop condition guarantees byte_pos < data.len()
            let b = unsafe { *self.data.get_unchecked(self.byte_pos) as u64 };

            self.buf |= b << self.count;

            self.count += 8;
            self.byte_pos += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_reader_basic() {
        let mut r = BitReader::new(&[0b0000_0011, 0b0000_0001]);
        assert_eq!(1, r.read_bit().unwrap());
        assert_eq!(1, r.read_bits(3).unwrap());
    }
}
