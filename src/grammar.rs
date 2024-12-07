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
    pub(crate) interlace_method: u8,
}

#[derive(Debug)]
pub struct Palette {
    pub(crate) palette: Vec<(u8, u8, u8)>,
}

#[derive(Debug)]
pub enum ColorType {
    Grayscale = 0,
    RGB = 2,
    Palette = 3,
    GrayscaleAlpha = 4,
    RGBA = 6,
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
