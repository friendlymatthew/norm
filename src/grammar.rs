use crate::ssim::LumaBuffer;
use anyhow::{bail, ensure, Result};
#[cfg(test)]
use std::io::Write;
use std::{borrow::Cow, collections::BTreeMap, slice::ChunksExact};
use std::{fs::File, io::Read, path::PathBuf};

#[derive(Debug)]
pub enum Chunk<'a> {
    ImageHeader(ImageHeader),
    Palette(ChunksExact<'a, u8>),
    ImageData(&'a [u8]),
    TextData(BTreeMap<&'a [u8], &'a [u8]>),
    Gamma(u32),
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
    pub(crate) const fn num_bytes_per_pixel(&self) -> usize {
        let bits_per_pixel = self.color_type.num_channels() * self.bit_depth;

        ((bits_per_pixel + 7) / 8) as usize
    }
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
    /// represents gamma * 100,000. `gamma` == 0 is SPECIAL.
    pub(crate) gamma: u32,
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

    pub const fn gamma(&self) -> u32 {
        self.gamma
    }

    /// The dimensions of the image (width, height).
    pub const fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub const fn color_type(&self) -> ColorType {
        self.color_type
    }

    pub fn to_rgb8(&self) -> Cow<'_, [u8]> {
        match self.color_type {
            ColorType::RGB => Cow::from(&self.pixel_buffer),
            ColorType::RGBA => {
                let b = self
                    .pixel_buffer
                    .chunks_exact(4)
                    .flat_map(|b| [b[0], b[1], b[2]])
                    .collect::<Vec<_>>();

                Cow::from(b)
            }
            ColorType::GrayscaleAlpha => {
                let b = self
                    .pixel_buffer
                    .chunks_exact(2)
                    .flat_map(|b| [b[0], b[0], b[0]])
                    .collect::<Vec<u8>>();

                Cow::from(b)
            }
            ColorType::Grayscale => {
                let b = self
                    .pixel_buffer
                    .iter()
                    .flat_map(|&y| [y, y, y])
                    .collect::<Vec<u8>>();

                Cow::from(b)
            }
            foreign => unimplemented!("{:?}", foreign),
        }
    }

    pub fn to_rgba8(&self) -> Cow<'_, [u8]> {
        match self.color_type {
            ColorType::RGBA => Cow::from(&self.pixel_buffer),
            ColorType::RGB => {
                let b = self
                    .pixel_buffer
                    .chunks_exact(3)
                    .flat_map(|b| [b[0], b[1], b[2], 0])
                    .collect::<Vec<_>>();

                Cow::from(b)
            }
            ColorType::Grayscale => {
                let b = self
                    .pixel_buffer
                    .iter()
                    .flat_map(|&y| [y, y, y, 0])
                    .collect::<Vec<_>>();

                Cow::from(b)
            }
            ColorType::GrayscaleAlpha => {
                let b = self
                    .pixel_buffer
                    .chunks_exact(2)
                    .flat_map(|b| [b[0], b[0], b[0], b[1]])
                    .collect::<Vec<_>>();

                Cow::from(b)
            }
            foreign => unimplemented!("{:?}", foreign),
        }
    }

    pub fn pixel_buffer(&self) -> Vec<u32> {
        match self.color_type {
            ColorType::RGB => self.rgb_buffer(),
            ColorType::RGBA => self.rgba_buffer(),
            ColorType::Grayscale => self.grayscale_buffer(),
            ColorType::GrayscaleAlpha => self.grayscale_alpha_buffer(),
            _ => todo!("What do other color type pixels look like?"),
        }
    }

    fn grayscale_buffer(&self) -> Vec<u32> {
        self.pixel_buffer
            .iter()
            .map(|&b| u32::from_be_bytes([0, b, b, b]))
            .collect::<Vec<u32>>()
    }

    fn grayscale_alpha_buffer(&self) -> Vec<u32> {
        self.pixel_buffer
            .chunks_exact(2)
            .map(|b| u32::from_be_bytes([b[1], b[0], b[0], b[0]]))
            .collect::<Vec<u32>>()
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

    /// Return luma values normalized to [0.0, 1.0] and the mean intensity.
    fn luma_buffer(&self) -> LumaBuffer {
        match self.color_type {
            ColorType::Grayscale => {
                let mut lumas = vec![0.0; self.pixel_buffer.len()];
                let mut mean_intensity = 0.0;

                self.pixel_buffer.iter().enumerate().for_each(|(i, &y)| {
                    lumas[i] = y as f32; // todo! What about other bit depths (not 8-bit)?
                    mean_intensity += lumas[i];
                });

                mean_intensity /= lumas.len() as f32;

                LumaBuffer::new(lumas, mean_intensity)
            }
            ColorType::GrayscaleAlpha => {
                let mut lumas = vec![0.0; self.pixel_buffer.len() / 2];
                let mut mean_intensity = 0.0;

                self.pixel_buffer
                    .chunks_exact(2)
                    .enumerate()
                    .for_each(|(i, b)| {
                        lumas[i] = b[0] as f32 / 255.0;
                        mean_intensity += lumas[i];
                    });

                mean_intensity /= lumas.len() as f32;
                LumaBuffer::new(lumas, mean_intensity)
            }
            ColorType::RGB => {
                let mut lumas = vec![0.0; self.pixel_buffer.len() / 3];
                let mut mean_intensity = 0.0;

                self.pixel_buffer
                    .chunks_exact(3)
                    .enumerate()
                    .for_each(|(i, rgb)| {
                        let (r, g, b) = (rgb[0] as f32, rgb[1] as f32, rgb[2] as f32);

                        lumas[i] = r * 0.29891 + g * 0.58661 + b * 0.11448;
                        mean_intensity += lumas[i];
                    });

                mean_intensity /= lumas.len() as f32;
                LumaBuffer::new(lumas, mean_intensity)
            }
            ColorType::RGBA => {
                let mut lumas = vec![0.0; self.pixel_buffer.len() / 4];
                let mut mean_intensity = 0.0;

                self.pixel_buffer
                    .chunks_exact(4)
                    .enumerate()
                    .for_each(|(i, rgb)| {
                        let (r, g, b) = (rgb[0] as f32, rgb[1] as f32, rgb[2] as f32);

                        lumas[i] = r * 0.29891 + g * 0.58661 + b * 0.11448;
                        mean_intensity += lumas[i];
                    });

                mean_intensity /= lumas.len() as f32;
                LumaBuffer::new(lumas, mean_intensity)
            }
            foreign => unimplemented!(
                "What does a luma buffer look like for palette: {:?}",
                foreign
            ),
        }
    }

    /// `compute_ssim` takes a full-reference image to calculate the global structural similarity index.
    /// A value closer to 1 indicates better image quality.
    pub fn compute_sim(&self, reference_image: &Self) -> Result<f32> {
        ensure!(
            self.dimensions() == reference_image.dimensions(),
            "Expect reference and test images to have identical dimensions."
        );

        let reference_luma_buffer = reference_image.luma_buffer();
        let test_luma_buffer = self.luma_buffer();

        assert_eq!(
            reference_luma_buffer.lumas.len(),
            test_luma_buffer.lumas.len()
        );

        Ok(test_luma_buffer.ssim(&reference_luma_buffer))
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn write_to_binary_blob(&self, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        file.write_all(&self.width.to_be_bytes())?;
        file.write_all(&self.height.to_be_bytes())?;
        file.write(&self.gamma.to_be_bytes())?;
        file.write_all(&[self.color_type as u8])?;
        file.write_all(&self.pixel_buffer)?;

        Ok(())
    }

    pub fn read_from_binary_blob(path: &PathBuf) -> Result<Self> {
        let mut file = File::open(path)?;

        let mut width = [0; 4];
        file.read_exact(&mut width)?;

        let mut height = [0; 4];
        file.read_exact(&mut height)?;

        let mut gamma = [0; 4];
        file.read_exact(&mut gamma)?;

        let mut color_type = [0];
        file.read_exact(&mut color_type)?;

        let mut pixel_buffer = Vec::new();
        file.read_to_end(&mut pixel_buffer)?;

        Ok(Self {
            width: u32::from_be_bytes(width),
            height: u32::from_be_bytes(height),
            gamma: u32::from_be_bytes(gamma),
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
