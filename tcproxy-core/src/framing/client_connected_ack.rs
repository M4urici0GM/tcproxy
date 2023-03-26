use std::io::Cursor;
use bytes::BufMut;
use crate::{Frame, FrameDecodeError};
use crate::framing::frame_types::CLIENT_CONNECTED_ACK;
use crate::framing::utils::assert_connection_type;
use crate::io::get_u16;

#[derive(Debug, PartialEq, Eq)]
pub struct ClientConnectedAck {
}

impl ClientConnectedAck {
    pub fn new() -> Self {
        Self {}
    }
}

impl Frame for ClientConnectedAck {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self : Sized {
        assert_connection_type(&get_u16(buffer)?, &CLIENT_CONNECTED_ACK)?;

        Ok(Self::new())
    }

    fn encode(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.put_u16(CLIENT_CONNECTED_ACK);

        vec
    }
}