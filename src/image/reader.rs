use crate::{
    image::grammar::{Image, ImageExt, ImageKind},
    jpeg::JpegDecoder,
    png::PngDecoder,
};
use anyhow::{bail, Result};
use std::path::Path;

#[derive(Debug)]
pub struct ImageReader;

impl ImageReader {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Image> {
        let image_path = path.as_ref();

        let image_kind = match image_path.extension().and_then(|e| e.to_str()) {
            Some("png") => ImageKind::Png,
            Some("jpeg") => ImageKind::Jpeg,
            _ => bail!("unsupported file type"),
        };

        let data = std::fs::read(path)?;

        let image: Box<dyn ImageExt> = match image_kind {
            ImageKind::Png => Box::new(PngDecoder::new(&data).decode()?),
            ImageKind::Jpeg => Box::new(JpegDecoder::new(&data).decode()?),
        };

        Ok(image)
    }
}
