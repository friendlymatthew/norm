use std::{
    fs::File,
    io::{Read, Write},
};

use anyhow::{bail, Result};

#[derive(Debug)]
pub enum Chunk<'a> {
    ImageHeader(ImageHeader),
    Palette(Palette),
    ImageData(&'a [u8]),
}

#[derive(Debug)]
pub struct ImageHeader {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) bit_depth: u8,
    pub(crate) color_type: ColorType,

    // Compression method should always be 0.
    pub(crate) _compression_method: u8,
    pub(crate) filter_method: u8,
    pub(crate) _interlace_method: bool,
}

impl ImageHeader {
    pub(crate) const fn num_bytes_per_pixel(&self) -> usize {
        (self.color_type.num_channels() * self.bit_depth) as usize / 8
    }
}

#[derive(Debug)]
pub struct Palette {
    pub(crate) palette: Vec<(u8, u8, u8)>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorType {
    Grayscale = 0,
    RGB = 2,
    Palette = 3,
    GrayscaleAlpha = 4,
    RGBA = 6,
}

impl ColorType {
    pub(crate) const fn num_channels(&self) -> u8 {
        match self {
            Self::Grayscale => 1,
            Self::RGB => 3,
            Self::Palette => 1,
            Self::GrayscaleAlpha => 2,
            Self::RGBA => 4,
        }
    }
}

impl TryFrom<u8> for ColorType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let val = match value {
            0 => Self::Grayscale,
            2 => Self::RGB,
            3 => Self::Palette,
            4 => Self::GrayscaleAlpha,
            6 => Self::RGBA,
            foreign => bail!("Unrecognized color type: {}", foreign),
        };

        Ok(val)
    }
}

#[derive(Debug)]
pub enum Filter {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

impl TryFrom<u8> for Filter {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let f = match value {
            0 => Self::None,
            1 => Self::Sub,
            2 => Self::Up,
            3 => Self::Average,
            4 => Self::Paeth,
            foreign => bail!("Unrecognized filter method: {}", foreign),
        };

        Ok(f)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Png {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) color_type: ColorType,
    pub(crate) pixel_buffer: Vec<u8>,
}

impl Png {
    pub const fn width(&self) -> u32 {
        self.width
    }

    pub const fn height(&self) -> u32 {
        self.height
    }

    pub const fn color_type(&self) -> ColorType {
        self.color_type
    }

    pub fn pixel_buffer(&self) -> Vec<u32> {
        match self.color_type {
            ColorType::RGB => self.rgb_buffer(),
            ColorType::RGBA => self.rgba_buffer(),
            _ => todo!("What do other color type pixels look like?"),
        }
    }

    fn rgb_buffer(&self) -> Vec<u32> {
        self.pixel_buffer
            .chunks_exact(3)
            .map(|b| u32::from_be_bytes([0, b[0], b[1], b[2]]))
            .collect::<Vec<u32>>()
    }

    fn rgba_buffer(&self) -> Vec<u32> {
        self.pixel_buffer
            .chunks_exact(4)
            .map(|b| u32::from_be_bytes([b[3], b[0], b[1], b[2]]))
            .collect::<Vec<u32>>()
    }

    pub fn write(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        file.write_all(&self.width.to_be_bytes())?;
        file.write_all(&self.height.to_be_bytes())?;
        file.write_all(&[self.color_type as u8])?;
        file.write_all(&self.pixel_buffer)?;

        Ok(())
    }

    pub fn read(path: &str) -> Result<Self> {
        let mut file = File::open(path)?;

        let mut width = [0; 4];
        file.read_exact(&mut width)?;

        let mut height = [0; 4];
        file.read_exact(&mut height)?;

        let mut color_type = [0];
        file.read_exact(&mut color_type)?;

        let mut pixel_buffer = Vec::new();
        file.read_to_end(&mut pixel_buffer)?;

        Ok(Self {
            width: u32::from_be_bytes(width),
            height: u32::from_be_bytes(height),
            color_type: color_type[0].try_into()?,
            pixel_buffer,
        })
    }
}

/* todo!("What would custom ZLib decompression look like?)
#[derive(Debug)]
pub struct ZLib {
    pub(crate) compression_method_flags: u8,
    pub(crate) additional_flags: u8,
    pub(crate) check_value: u32,
}

impl ZLib {
    pub fn compression_method(&self) -> u8 {
        self.compression_method_flags & 0b1111
    }

    pub fn compression_info(&self) -> u8 {
        (self.compression_method_flags & 0b1111_0000) >> 4
    }

    pub fn flag_check(&self) -> u8 {
        self.additional_flags & 0b1_1111
    }

    pub fn preset_dictionary(&self) -> bool {
        self.additional_flags & 0b10_0000 != 0
    }

    pub fn compression_level(&self) -> u8 {
        (self.additional_flags & 0b1100_0000) >> 6
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Block {
    NoCompression = 0b00,
    FixedHuffmanCodes = 0b01,
    DynamicHuffmanCodes = 0b10,
    Reserved = 0b11,
}

impl TryFrom<usize> for Block {
    type Error = anyhow::Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let bt = match value {
            0b00 => Self::NoCompression,
            0b01 => Self::FixedHuffmanCodes,
            0b10 => Self::DynamicHuffmanCodes,
            0b11 => Self::Reserved,
            foreign => bail!("Unrecognized block type: {}", foreign),
        };

        Ok(bt)
    }
}
*/
