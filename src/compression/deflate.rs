use std::sync::LazyLock;

use anyhow::{bail, ensure, Result};

use crate::{compression::huffman::HuffmanDecoder, compression::BitReader};

const LENGTH_BASE: [usize; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [usize; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];

const DIST_BASE: [usize; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [usize; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

const CL_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

static FIXED_CODES: LazyLock<(HuffmanDecoder, HuffmanDecoder)> = LazyLock::new(fixed_codes);

pub fn decode(input: &[u8]) -> Result<Vec<u8>> {
    let mut reader = BitReader::new(input);
    let mut out = Vec::new();

    loop {
        let b_final = reader.read_bit()?;
        let b_type = reader.read_bits(2)?;

        match b_type {
            0b00 => {
                // no compression
                reader.align_to_byte();

                let header = reader.read_array::<4>()?;

                let len = u16::from_le_bytes([header[0], header[1]]);
                let nlen = u16::from_le_bytes([header[2], header[3]]);
                ensure!(
                    len ^ nlen == 0xFFFF,
                    "stored block: NLEN is not the complement of LEN"
                );

                out.extend_from_slice(reader.read_bytes(len as usize)?);
            }
            0b01 => {
                // compressed with fixed huffman codes
                let (lit, dist) = &*FIXED_CODES;
                inflate_block(&mut reader, &mut out, lit, dist)?;
            }
            0b10 => {
                // compressed with dynamic huffman codes
                let (lit, dist) = read_dynamic_codes(&mut reader)?;
                inflate_block(&mut reader, &mut out, &lit, &dist)?;
            }
            0b11 => {
                bail!("invalid block type 3");
            }
            _ => unreachable!(),
        }

        if b_final == 1 {
            break;
        }
    }

    Ok(out)
}

fn inflate_block(
    reader: &mut BitReader,
    out: &mut Vec<u8>,
    lit: &HuffmanDecoder,
    dist: &HuffmanDecoder,
) -> Result<()> {
    loop {
        let sym = lit.decode(reader)? as usize;

        match sym {
            0..=255 => out.push(sym as u8),
            256 => return Ok(()),
            257..=285 => {
                let i = sym - 257;
                let length = LENGTH_BASE[i] + reader.read_bits(LENGTH_EXTRA[i])?;

                let dist_sym = dist.decode(reader)? as usize;
                ensure!(dist_sym < 30, "invalid distance symbol {dist_sym}");
                let distance = DIST_BASE[dist_sym] + reader.read_bits(DIST_EXTRA[dist_sym])?;

                ensure!(distance <= out.len(), "distance too far back");

                let start = out.len() - distance;

                out.reserve(length);

                for i in 0..length {
                    out.push(out[start + i]);
                }
            }
            _ => bail!("invalid literal or length symbol {sym}"),
        }
    }
}

fn fixed_codes() -> (HuffmanDecoder, HuffmanDecoder) {
    let mut lit = [0u8; 288];
    lit[0..144].fill(8);
    lit[144..256].fill(9);
    lit[256..280].fill(7);
    lit[280..288].fill(8);

    (
        HuffmanDecoder::try_new(&lit).unwrap(),
        HuffmanDecoder::try_new(&[5; 32]).unwrap(),
    )
}

fn read_dynamic_codes(reader: &mut BitReader) -> Result<(HuffmanDecoder, HuffmanDecoder)> {
    let hlit = reader.read_bits(5)? + 257;
    let hdist = reader.read_bits(5)? + 1;
    let hclen = reader.read_bits(4)? + 4;

    ensure!(hlit < 287);
    ensure!(hdist < 31);

    let mut cl_lengths = [0u8; 19];
    for &sym in &CL_ORDER[..hclen] {
        cl_lengths[sym] = reader.read_bits(3)? as u8;
    }
    let cl = HuffmanDecoder::try_new(&cl_lengths)?;

    let total = hlit + hdist;
    let mut lengths = [0u8; 286 + 30];
    let mut n = 0;

    while n < total {
        let (len, repeat) = match cl.decode(reader)? {
            sym @ 0..=15 => (sym as u8, 1),
            16 => {
                ensure!(n > 0, "repeat with no previous length");
                (lengths[n - 1], 3 + reader.read_bits(2)?)
            }
            17 => (0, 3 + reader.read_bits(3)?),
            18 => (0, 11 + reader.read_bits(7)?),
            _ => unreachable!(),
        };

        ensure!(
            n + repeat <= total,
            "repeat past the end of the code lengths"
        );
        lengths[n..n + repeat].fill(len);
        n += repeat;
    }

    let (lit_lengths, dist_lengths) = lengths[..total].split_at(hlit);
    ensure!(lit_lengths[256] != 0, "missing end-of-block code");

    Ok((
        HuffmanDecoder::try_new(lit_lengths)?,
        HuffmanDecoder::try_new(dist_lengths)?,
    ))
}
