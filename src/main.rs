use anyhow::{anyhow, Result};
use normeditor::{renderer, ImageReader};
use pollster::block_on;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let image_path = args
        .next()
        .ok_or_else(|| anyhow!("Failed to read image path"))?;

    let image = ImageReader::from_path(&image_path)?;

    let _ = block_on(renderer::run(image));

    Ok(())
}
