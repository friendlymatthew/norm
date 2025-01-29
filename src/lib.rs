#![warn(clippy::nursery)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub use decoder::*;
pub use grammar::*;
pub mod renderer;
pub mod ssim;
pub mod test_file_parser;

mod crc32;
mod decoder;
mod grammar;
mod interlace;
pub mod util;
