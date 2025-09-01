use crate::qoi::grammar::Qoi;
use anyhow::Result;

#[derive(Debug)]
pub struct QoiDecoder<'a> {
    cursor: usize,
    data: &'a [u8],
}

impl<'a> QoiDecoder<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        Self { cursor: 0, data }
    }

    pub fn decode(&mut self) -> Result<Qoi> {
        todo!()
    }
}
