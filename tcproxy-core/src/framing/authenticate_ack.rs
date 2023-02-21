use std::io::Cursor;
use crate::{Frame, FrameDecodeError};

#[derive(Debug, PartialEq)]
pub struct AuthenticateAck {

}

impl Frame for AuthenticateAck {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self: Sized {
        todo!()
    }

    fn encode(&self) -> Vec<u8> {
        todo!()
    }
}