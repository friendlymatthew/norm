mod bitreader;
pub mod deflate;
mod huffman;
mod zlib;

use bitreader::*;
pub use zlib::ZlibDecoder;
