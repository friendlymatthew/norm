#![warn(clippy::nursery)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub use decoder::*;
pub use grammar::*;
pub mod renderer;
pub mod test_file_parser;

mod decoder;
mod grammar;
mod texture;

mod crc32;
mod interlace;
