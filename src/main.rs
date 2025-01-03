use anyhow::{anyhow, Result};
use minifb::{Window, WindowOptions};
use png::Decoder;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let image_path = args
        .next()
        .ok_or_else(|| anyhow!("Failed to read image path"))?;

    let content = std::fs::read(image_path)?;
    let mut decoder = Decoder::new(&content);
    let png = decoder.decode()?;

    let (width, height) = png.dimension();

    let mut window = Window::new(
        "PNG renderer",
        width as usize,
        height as usize,
        WindowOptions::default(),
    )?;

    while window.is_open() {
        window.update_with_buffer(&png.pixel_buffer(), width as usize, height as usize)?;
    }

    Ok(())
}
