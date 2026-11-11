#![warn(clippy::nursery)]

#[doc(hidden)]
pub use anyhow;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod compression;
pub mod font;
pub mod image;
pub mod jpeg;
pub mod png;
pub mod renderer;

pub mod event_log;
pub(crate) mod impl_read;
pub mod test_file_parser;

pub use image::*;
