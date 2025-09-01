#![warn(clippy::nursery)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod font;
pub mod image;
pub mod jpeg;
pub mod png;
pub mod qoi;
pub mod renderer;
pub mod util;
