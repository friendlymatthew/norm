use anyhow::Result;
use minifb::{Window, WindowOptions};

use png::decoder::Decoder;

fn main() -> Result<()> {
    let content = std::fs::read("./tests/potatoe.png")?;
    let mut decoder = Decoder::new(&content);
    let png = decoder.decode()?;

    let mut window = Window::new(
        "Potatoe",
        png.width() as usize,
        png.height() as usize,
        WindowOptions::default(),
    )?;

    while window.is_open() {
        window.update_with_buffer(
            &png.rgba_buffer(),
            png.width() as usize,
            png.height() as usize,
        )?;
    }

    Ok(())
}
