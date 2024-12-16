// A parser to parse the file names in PngSuite.
// http://www.schaik.com/pngsuite/

use std::{os::unix::ffi::OsStrExt, path::PathBuf};

use anyhow::{anyhow, bail, ensure, Result};

use crate::ColorType;

#[derive(Debug)]
pub enum TestFeature {
    Basic,
    Interlacing,
    OddSizes,
    BackgroundColors,
    Transparency,
    GammaValues,
    ImageFiltering,
    AdditionalPalettes,
    AncillaryChunks,
    ChunkOrder,
    ZlibCompression,
    CorruptedFiles,
}

#[derive(Debug)]
pub struct TestFile {
    pub test_feature: TestFeature,
    pub description: String,
    pub interlaced: bool,
    pub color_type: ColorType,
    pub bit_depth: u8,
}

#[derive(Debug, Default)]
pub struct TestFileParser;

impl TestFileParser {
    fn parse_interlace(&self, i: u8) -> Result<bool> {
        match i {
            b'n' => Ok(false),
            b'i' => Ok(true),
            foreign => bail!("Unexpected interlace: {foreign}"),
        }
    }

    fn parse_color_type(&self, ct: u8, ct_desc: u8) -> Result<ColorType> {
        match (ct, ct_desc) {
            (b'0', b'g') => Ok(ColorType::Grayscale),
            (b'2', b'c') => Ok(ColorType::RGB),
            (b'3', b'p') => Ok(ColorType::Palette),
            (b'4', b'a') => Ok(ColorType::GrayscaleAlpha),
            (b'6', b'a') => Ok(ColorType::RGBA),
            foreign => bail!("Unexpected color type: {foreign:?}"),
        }
    }

    fn parse_bit_depth(&self, bd: &[u8]) -> Result<u8> {
        let res = match bd {
            b"01" => 1,
            b"02" => 2,
            b"04" => 4,
            b"08" => 8,
            b"16" => 16,
            foreign => bail!("Unexpected color type: {foreign:?}"),
        };

        Ok(res)
    }

    pub fn parse(&self, file_path: PathBuf) -> Result<TestFile> {
        let file_string = file_path
            .file_stem()
            .ok_or_else(|| anyhow!("Failed to get file name"))?;

        let file_path = file_string.as_bytes();

        ensure!(
            file_path.len() == 8,
            "Invalid file name length. Expected 8 characters. File path: {:?}",
            file_string
        );

        let test_file = match file_path {
            &[b'b', b'a', b's', interlace, color_type, color_type_desc, ref rest @ ..] => {
                let description = "A basic PNG format";
                let interlaced = self.parse_interlace(interlace)?;

                let color_type = self.parse_color_type(color_type, color_type_desc)?;

                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::Basic,
                    description: description.into(),
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            &[b'b', b'g', bg_ty, interlace, color_type, color_type_desc, ref rest @ ..] => {
                let description = match bg_ty {
                    b'a' => "alpha + no background",
                    b'w' => "alpha + white background",
                    b'g' => "alpha + gray background",
                    b'b' => "alpha + black background",
                    b'y' => "alpha + yellow background",
                    foreign => bail!("Foreign code: {foreign}"),
                };
                let interlaced = self.parse_interlace(interlace)?;

                let color_type = self.parse_color_type(color_type, color_type_desc)?;

                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::BackgroundColors,
                    description: description.into(),
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            &[b't', t_ty_1, t_ty_2, interlace, color_type, color_type_desc, ref rest @ ..] => {
                let description = match &[t_ty_1, t_ty_2] {
                    b"bw" => "transparent + white background",
                    b"bg" => "transparent + gray background",
                    b"bb" => "transparent + black background",
                    b"br" => "transparent + red background chunk",
                    b"by" => "transparent + yellow background",
                    b"p0" => "not transparent for reference",
                    b"p1" => "transparent, but no background chunk",
                    b"m3" => "multiple levels of transparency, 3 entries",
                    foreign => bail!("Foreign code: {foreign:?}"),
                };
                let interlaced = self.parse_interlace(interlace)?;

                let color_type = self.parse_color_type(color_type, color_type_desc)?;

                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::Transparency,
                    description: description.into(),
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            &[b'g', g_ty_1, g_ty_2, interlace, color_type, color_type_desc, ref rest @ ..] => {
                let description = match &[g_ty_1, g_ty_2] {
                    b"03" => "file-gamma = 0.35, for display with gamma = 2.8",
                    b"04" => "file-gamma = 0.45, for display with gamma = 2.2 (PC)",
                    b"05" => "file-gamma = 0.55, for display with gamma = 1.8 (Mac)",
                    b"07" => "file-gamma = 0.70, for display with gamma = 1.4",
                    b"10" => "file-gamma = 1.00, for display with gamma = 1.0 (NeXT)",
                    b"25" => "file-gamma = 2.50, for display with gamma = 0.4",
                    foreign => bail!("Foreign code: {foreign:?}"),
                };

                let interlaced = self.parse_interlace(interlace)?;

                let color_type = self.parse_color_type(color_type, color_type_desc)?;

                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::GammaValues,
                    description: description.into(),
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            &[b'f', f_ty_1, f_ty_2, interlace, color_type, color_type_desc, ref rest @ ..] => {
                let description = match &[f_ty_1, f_ty_2] {
                    b"00" => "filter-type 0",
                    b"01" => "filter-type 1",
                    b"02" => "filter-type 2",
                    b"03" => "filter-type 3",
                    b"04" => "filter-type 4",
                    b"99" => "filter changing per scanline",
                    foreign => bail!("Foreign code: {foreign:?}"),
                };

                let interlaced = self.parse_interlace(interlace)?;

                let color_type = self.parse_color_type(color_type, color_type_desc)?;

                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::ImageFiltering,
                    description: description.into(),
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            &[b'c', c_kind, c_desc, interlace, color_type, color_type_desc, ref rest @ ..] => {
                let description = match &[c_kind, c_desc] {
                    b"s3" => "3/13 significant bits",
                    b"s5" => "5 significant bits",
                    b"s8" => "8 significant bits (reference)",
                    b"df" => "physical pixel dimensions, 8x32 flat pixels",
                    b"dh" => "physical pixel dimensions, 32x8 high pixels",
                    b"ds" => "physical pixel dimensions, 8x8 square pixels",
                    b"du" => "physical pixel dimensions, with unit specifier",
                    b"cw" => "primary chromaticities and white point",
                    b"h1" => "histogram 15 colors",
                    b"h2" => "histogram 256 colors",
                    b"m7" => "modification time, 01-jan-1970",
                    b"m9" => "modification time, 31-dec-1999",
                    b"m0" => "modification time, 01-jan-2000",
                    b"t0" => "no textual data",
                    b"t1" => "with textual data",
                    b"tz" => "with compressed textual data",
                    b"te" => "UTF-8 international text - english",
                    b"tf" => "UTF-8 international text - finnish",
                    b"tg" => "UTF-8 international text - greek",
                    b"th" => "UTF-8 international text - hindi",
                    b"tj" => "UTF-8 international text - japanese",
                    foreign => bail!("Foreign code: {foreign:?}"),
                };

                let interlaced = self.parse_interlace(interlace)?;

                let color_type = self.parse_color_type(color_type, color_type_desc)?;

                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::AncillaryChunks,
                    description: description.into(),
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            &[b'e', b'x', b'i', b'f', color_type, color_type_desc, ref rest @ ..] => {
                let description = "with exif data";

                let color_type = self.parse_color_type(color_type, color_type_desc)?;

                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::AncillaryChunks,
                    description: description.into(),
                    interlaced: false,
                    color_type,
                    bit_depth,
                }
            }
            &[b'o', b'i', image_code, interlace, color_type, color_type_desc, ref rest @ ..] => {
                let description = match image_code - b'0' {
                    1 => "mother image with 1 idat-chunk",
                    2 => "image with 2 idat-chunks",
                    4 => "image with 4 unequal sized idat-chunks",
                    9 => "all idat-chunks of length one",
                    foreign => bail!("Foreign code: {foreign:?}"),
                };
                let interlaced = self.parse_interlace(interlace)?;

                let color_type = self.parse_color_type(color_type, color_type_desc)?;

                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::ZlibCompression,
                    description: description.into(),
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            &[b'z', b'0', z_ty, interlace, color_type, color_type_desc, ref rest @ ..] => {
                let description = match z_ty - b'0' {
                    0 => "zlib compression level 0 - none",
                    3 => "zlib compression level 1",
                    6 => "zlib compression level 2 - default",
                    9 => "zlib compression level 9 - maximum",
                    foreign => bail!("Foreign code: {foreign:?}"),
                };
                let interlaced = self.parse_interlace(interlace)?;
                let color_type = self.parse_color_type(color_type, color_type_desc)?;
                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::ZlibCompression,
                    description: description.into(),
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            &[b's', dim_1, dim_2, interlace, color_type, color_type_desc, ref rest @ ..] => {
                dbg!(dim_1);
                let dim = (dim_1 - b'0') * 10 + (dim_2 - b'0');
                let description = format!("{dim}x{dim} paletted file");
                let interlaced = self.parse_interlace(interlace)?;
                let color_type = self.parse_color_type(color_type, color_type_desc)?;
                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::OddSizes,
                    description,
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            &[b'x', c_ty_1, c_ty_2, interlace, color_type, color_type_desc, ref rest @ ..] => {
                let description = match &[c_ty_1, c_ty_2] {
                    b"s1" => "signature byte 1 MSBit reset to zero",
                    b"s2" => "signature byte 2 is a 'Q'",
                    b"s4" => "signature byte 4 lowercase",
                    b"s7" => "7th byte a space instead of control-Z",
                    b"cr" => "added cr bytes",
                    b"lf" => "added lf bytes",
                    b"hd" => "incorrect IHDR checksum",
                    b"c1" => "color type 1",
                    b"c9" => "color type 9",
                    b"d0" => "bit-depth 0",
                    b"d3" => "bit-depth 3",
                    b"d9" => "bit-depth 99",
                    b"dt" => "missing IDAT chunk",
                    b"cs" => "incorrect IDAT checksum",
                    foreign => bail!("Unexpected code: {foreign:?}"),
                };

                let interlaced = self.parse_interlace(interlace)?;
                let color_type = self.parse_color_type(color_type, color_type_desc)?;
                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::CorruptedFiles,
                    description: description.into(),
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            &[b'p', p_ty_1, p_ty_2, interlace, color_type, color_type_desc, ref rest @ ..] => {
                let description = match &[p_ty_1, p_ty_2] {
                    b"p0" => "six-cube palette chunk",
                    b"s1" => "six-cube suggested palette (1 byte)",
                    b"s2" => "six-cube suggested palette (2 bytes)",
                    foreign => bail!("Unexpected code: {foreign:?}"),
                };

                let interlaced = self.parse_interlace(interlace)?;
                let color_type = self.parse_color_type(color_type, color_type_desc)?;
                let bit_depth = self.parse_bit_depth(rest)?;

                TestFile {
                    test_feature: TestFeature::CorruptedFiles,
                    description: description.into(),
                    interlaced,
                    color_type,
                    bit_depth,
                }
            }
            foreign => bail!("Unexpected code: {foreign:?}"),
        };

        Ok(test_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_files() -> Result<()> {
        let dir_entry = std::fs::read_dir("./tests")?;

        let test_file_parser = TestFileParser::default();

        dir_entry.for_each(|entry| {
            let entry = entry.ok().unwrap();
            let path = entry.path();
            let tf = test_file_parser.parse(entry.path());

            assert!(tf.is_ok(), "{:?}, {:?}", &path.file_name().unwrap(), tf);
        });

        Ok(())
    }
}
