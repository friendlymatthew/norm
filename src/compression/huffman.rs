use anyhow::{bail, ensure, Result};

use super::bitreader::BitReader;

// note the deflate code lengths never exceed 15 bits
const MAX_BITS: usize = 15;

#[derive(Debug)]
pub struct HuffmanDecoder {
    counts: [u16; MAX_BITS + 1],
    symbols: Vec<u16>,
}

impl HuffmanDecoder {
    pub fn try_new(lengths: &[u8]) -> Result<Self> {
        let mut counts = [0u16; MAX_BITS + 1];
        for &l in lengths {
            counts[l as usize] += 1;
        }
        counts[0] = 0;

        let mut offsets = [0u16; MAX_BITS + 1];
        for len in 1..MAX_BITS {
            offsets[len + 1] = offsets[len] + counts[len];
        }

        let mut left: i32 = 1;
        (1..=MAX_BITS).try_for_each(|len| {
            left <<= 1;
            left -= counts[len] as i32;
            ensure!(left >= 0, "over-subscribed code lengths");

            Ok(())
        })?;

        let mut symbols = vec![0; counts.iter().map(|&c| c as usize).sum()];
        for (sym, &len) in lengths.iter().enumerate() {
            if len != 0 {
                let slot = &mut offsets[len as usize];
                symbols[*slot as usize] = sym as u16;
                *slot += 1;
            }
        }

        Ok(Self { counts, symbols })
    }

    pub fn decode(&self, reader: &mut BitReader) -> Result<u16> {
        let mut code = 0;
        let mut first = 0;
        let mut index = 0;

        for len in 1..=MAX_BITS {
            code |= reader.read_bit()?;
            let count = self.counts[len] as usize;

            if code < first + count {
                return Ok(self.symbols[index + (code - first)]);
            }

            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }

        bail!("invalid huffman code")
    }
}
