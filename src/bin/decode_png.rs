use anyhow::{anyhow, Result};
use normeditor::png::PngDecoder;
#[cfg(feature = "time")]
use normeditor::util::event_log::{log_event, Event};
#[cfg(feature = "time")]
use std::time::Instant;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let image_path = args
        .next()
        .ok_or_else(|| anyhow!("Failed to read image path"))?;
    //
    // let image_kind = args.next().and_then(|image_kind| {
    //     match image_kind.as_bytes() {
    //         b"jpeg" => {}
    //         b"png" => {}
    //     }
    // });

    let content = std::fs::read(image_path)?;

    let mut decoder = PngDecoder::new(&content);

    #[cfg(feature = "time")]
    let a = Instant::now();
    let _ = decoder.decode()?;

    #[cfg(feature = "time")]
    log_event("", Event::TotalElapsed, Some(a.elapsed()));

    Ok(())
}
