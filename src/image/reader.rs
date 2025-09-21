use crate::{
    image::grammar::{Image, ImageExt, ImageKind},
    jpeg::JpegDecoder,
    png::PngDecoder,
};
use anyhow::Result;
use std::path::Path;

#[derive(Debug)]
pub struct ImageReader;

impl ImageReader {
    pub fn read_from_path(path: impl AsRef<Path>, image_kind: Option<ImageKind>) -> Result<Image> {
        let data = std::fs::read(path)?;

        let image_kind = image_kind.expect("how do you infer which image decoder to run?");

        let image: Box<dyn ImageExt> = match image_kind {
            ImageKind::Png => Box::new(PngDecoder::new(&data).decode()?),
            ImageKind::Jpeg => Box::new(JpegDecoder::new(&data).decode()?),
        };

        Ok(image)
    }
}
