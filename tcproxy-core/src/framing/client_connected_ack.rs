use std::io::Cursor;
use bytes::BufMut;
use crate::{Frame, FrameDecodeError};
use crate::framing::frame_types::CLIENT_CONNECTED_ACK;
use crate::framing::utils::assert_connection_type;
use crate::io::{get_u16, get_u8};

#[derive(Debug, PartialEq, Eq)]
pub struct ClientConnectedAck {
    port: u16
}

impl ClientConnectedAck {
    pub fn new(port: &u16) -> Self {
        Self {
            port: *port,
        }
    }

    pub fn port(&self) -> &u16 {
        &self.port
    }
}

impl Frame for ClientConnectedAck {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self : Sized {
        assert_connection_type(&get_u8(buffer)?, &CLIENT_CONNECTED_ACK)?;

        let port = get_u16(buffer)?;
        Ok(Self { port })
    }

    fn encode(&self) -> Vec<u8> {
        let mut vec = Vec::new();

        vec.put_u8(CLIENT_CONNECTED_ACK);
        vec.put_u16(self.port);

        vec
    }
}