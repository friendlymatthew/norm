#![allow(clippy::all)]

pub mod grammar;
mod outline;
mod parser;
pub mod shaper;

pub use outline::{outline, Contour, Point, Segment};
pub use parser::*;
