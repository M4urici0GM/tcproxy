use crate::framing::frame_types::CLIENT_CONNECTED_ACK;
use crate::framing::utils::assert_connection_type;
use crate::io::get_u16;
use crate::{Frame, FrameDecodeError};
use bytes::BufMut;
use std::io::Cursor;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ClientConnectedAck {
    listening_port: u16
}

impl ClientConnectedAck {
    pub fn new(port: &u16) -> Self {
        Self {
            listening_port: *port
        }
    }
}

impl Frame for ClientConnectedAck {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError>
    where
        Self: Sized,
    {
        assert_connection_type(&get_u16(buffer)?, &CLIENT_CONNECTED_ACK)?;
        let listening_port = get_u16(buffer)?;

        Ok(Self::new(&listening_port))
    }

    fn encode(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.put_u16(CLIENT_CONNECTED_ACK);
        vec.put_u16(self.listening_port);

        vec
    }
}
