use std::io::Cursor;
use crate::{Frame, FrameDecodeError};

#[derive(Debug, PartialEq, Eq)]
pub struct ClientConnected;

impl ClientConnected {
    pub fn new() -> Self {
        Self {}
    }
}

impl Frame for ClientConnected {
    fn decode(_buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self: Sized {
        return Ok(Self);
    }

    fn encode(&self) -> Vec<u8> {
        return vec![b'*'];
    }
}