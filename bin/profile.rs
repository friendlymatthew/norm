use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;
use png::decoder::Decoder;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    image_path: PathBuf,
}

fn main() -> Result<()> {
    let Args { image_path } = Args::parse();

    let image_path = image_path
        .to_str()
        .ok_or(anyhow!("Failed to find file {:?} to render.", image_path))?;

    let content = std::fs::read(image_path)?;
    let mut decoder = Decoder::new(&content);
    let _ = decoder.decode()?;

    Ok(())
}
