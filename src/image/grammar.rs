use anyhow::bail;
use std::borrow::Cow;

#[derive(Debug)]
pub enum ImageKind {
    Png,
    Jpeg,
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

    fn try_from(value: u8) -> anyhow::Result<Self, Self::Error> {
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

pub type Image = Box<dyn ImageExt>;

pub trait ImageExt {
    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    fn gamma(&self) -> u32;

    fn color_type(&self) -> ColorType;

    fn rgb8(&self) -> Cow<'_, [u8]>;

    fn rgba8(&self) -> Cow<'_, [u8]>;

    fn bitmap(&self) -> Cow<'_, [u32]>;
}
