use anyhow::bail;

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
    pub(crate) compression_method: u8,
    pub(crate) filter_method: u8,
    pub(crate) interlace_method: bool,
}

impl ImageHeader {
    pub(crate) fn num_bytes_per_pixel(&self) -> usize {
        (self.color_type.num_channels() * self.bit_depth) as usize / 8
    }
}

#[derive(Debug)]
pub struct Palette {
    pub(crate) palette: Vec<(u8, u8, u8)>,
}

#[derive(Debug, Copy, Clone)]
pub enum ColorType {
    Grayscale = 0,
    RGB = 2,
    Palette = 3,
    GrayscaleAlpha = 4,
    RGBA = 6,
}

impl ColorType {
    pub(crate) fn num_channels(&self) -> u8 {
        match self {
            ColorType::Grayscale => 1,
            ColorType::RGB => 3,
            ColorType::Palette => 1,
            ColorType::GrayscaleAlpha => 2,
            ColorType::RGBA => 4,
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
            0 => Filter::None,
            1 => Filter::Sub,
            2 => Filter::Up,
            3 => Filter::Average,
            4 => Filter::Paeth,
            foreign => bail!("Unrecognized filter method: {}", foreign),
        };

        Ok(f)
    }
}

#[derive(Debug)]
pub struct Png {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) color_type: ColorType,
    pub(crate) image_data: Vec<u8>,
}

impl Png {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn color_type(&self) -> ColorType {
        self.color_type
    }

    pub fn rgba_buffer(&self) -> Vec<u32> {
        self.image_data
            .chunks_exact(4)
            .map(|b| u32::from_be_bytes([b[3], b[0], b[1], b[2]]))
            .collect::<Vec<u32>>()
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
