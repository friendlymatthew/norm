use crate::image::grammar::{ColorType, ImageExt};
use anyhow::bail;
use std::{borrow::Cow, ops::RangeInclusive};

pub type Marker = u16;

#[derive(Debug)]
pub struct ApplicationHeader {
    pub version: (u8, u8),
    pub unit: u8,
    pub density: (u16, u16),
    pub thumbnail: (u8, u8),
}

#[derive(Debug, Clone, Copy)]
pub enum Precision {
    Eight = 1,
    Sixteen = 2,
}

impl From<bool> for Precision {
    fn from(value: bool) -> Self {
        if value {
            Self::Sixteen
        } else {
            Self::Eight
        }
    }
}

#[derive(Debug)]
pub struct QuantizationTable {
    pub flag: u8,
    pub table_elements: [u16; QuantizationTable::NUM_ELEMENTS],
}

impl QuantizationTable {
    // The number of elements per quantization table
    pub const NUM_ELEMENTS: usize = 64;

    pub fn precision(&self) -> Precision {
        (self.flag >> 4 == 1).into()
    }

    pub const fn table_identifier(&self) -> u8 {
        self.flag & 0b1111
    }
}

pub enum HuffmanTableClass {
    AC,
    DC,
}

#[derive(Debug)]
pub struct HuffmanTable {
    pub flag: u8,
    pub code_lengths: [u8; 16],
    pub values: Vec<u8>,
}

impl HuffmanTable {
    pub const fn table_class(&self) -> HuffmanTableClass {
        if (self.flag >> 4) == 1 {
            return HuffmanTableClass::AC;
        }

        HuffmanTableClass::DC
    }

    pub const fn table_identifier(&self) -> u8 {
        self.flag & 0b1111
    }
}

#[derive(Debug)]
pub enum EncodingProcess {
    BaselineDCT = 0,
    HuffmanExtendedSequentialDCT = 1,
    HuffmanProgressiveDCT = 2,
    HuffmanLossless = 3,
    ArithmeticExtendedSequentialDCT = 9,
    ArithmeticProgressiveDCT = 10,
    ArithmeticLossless = 11,
}

impl TryFrom<u8> for EncodingProcess {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let res = match value {
            0 => Self::BaselineDCT,
            1 => Self::HuffmanExtendedSequentialDCT,
            2 => Self::HuffmanProgressiveDCT,
            3 => Self::HuffmanLossless,
            9 => Self::ArithmeticExtendedSequentialDCT,
            10 => Self::ArithmeticProgressiveDCT,
            11 => Self::ArithmeticLossless,
            foreign => bail!("Encountered foreign encoding process: {foreign}"),
        };

        Ok(res)
    }
}

#[derive(Debug)]
pub struct StartOfFrame {
    pub encoding_process: EncodingProcess,
    pub sample_precision: u8,
    pub lines: u16,
    pub samples_per_line: u16,
    pub components: Vec<Component>,
}

#[derive(Debug)]
pub struct Component {
    pub identifier: u8,
    pub sampling_factor: u8,
    pub quantization_table_destination_selector: u8,
}

#[derive(Debug)]
pub struct StartOfScan {
    pub components: Vec<(u8, u8)>,
    pub spectral_select: RangeInclusive<u8>,
    pub approximation: u8,
}

#[derive(Debug)]
pub struct JFIF<'a> {
    pub application_header: ApplicationHeader,
    pub quantization_tables: Vec<QuantizationTable>,
    pub huffman_tables: Vec<HuffmanTable>,
    pub start_of_frame: StartOfFrame,
    pub start_of_scan: StartOfScan,
    pub image_data: &'a [u8],
}

#[derive(Debug)]
pub struct Jpeg {}

impl ImageExt for Jpeg {
    fn width(&self) -> u32 {
        todo!()
    }

    fn height(&self) -> u32 {
        todo!()
    }

    fn gamma(&self) -> u32 {
        todo!()
    }

    fn color_type(&self) -> ColorType {
        todo!()
    }

    fn rgb8(&self) -> Cow<'_, [u8]> {
        todo!()
    }

    fn rgba8(&self) -> Cow<'_, [u8]> {
        todo!()
    }

    fn bitmap(&self) -> Cow<'_, [u32]> {
        todo!()
    }
}
