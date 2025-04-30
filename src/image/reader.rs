use crate::{
    image::grammar::{
        Image,
        ImageExt,
        ImageKind,
    },
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

        let image: Box<dyn ImageExt> = Box::new(match image_kind {
            ImageKind::Png => PngDecoder::new(&data).decode()?,
            ImageKind::Jpeg => {
                todo!();
            }
        });

        Ok(image)
    }
}
