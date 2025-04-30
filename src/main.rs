use anyhow::{
    anyhow,
    Result,
};
use iris::{
    image::{
        grammar::ImageKind,
        ImageReader,
    },
    renderer,
};
use pollster::block_on;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let image_path = args
        .next()
        .ok_or_else(|| anyhow!("Failed to read image path"))?;

    let image = ImageReader::read_from_path(&image_path, Some(ImageKind::Png))?;

    let _ = block_on(renderer::run(image));

    Ok(())
}
