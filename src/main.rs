use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use png::Engine;
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    image_path: PathBuf,
}

fn main() -> Result<()> {
    let Args { image_path } = Args::parse();
    let content = std::fs::read(image_path)?;

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut engine = Engine::new(&content)?;
    event_loop.run_app(&mut engine)?;

    Ok(())
}
