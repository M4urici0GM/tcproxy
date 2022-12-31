use std::io::Cursor;
use crate::{Frame, FrameDecodeError};

#[derive(Debug, PartialEq, Eq)]
pub struct Ping;

impl Ping {
    pub fn new() -> Self {
        Self {}
    }
}

impl Frame for Ping {
    fn decode(_: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self: Sized {
        return Ok(Self);
    }

    fn encode(&self) -> Vec<u8> {
        return vec![b'-'];
    }
}