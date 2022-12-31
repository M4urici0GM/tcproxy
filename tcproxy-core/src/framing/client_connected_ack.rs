use std::io::Cursor;
use bytes::BufMut;
use crate::{Frame, FrameDecodeError};
use crate::io::get_u16;

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
        let port = get_u16(buffer)?;
        Ok(Self { port })
    }

    fn encode(&self) -> Vec<u8> {
        let mut vec = Vec::new();

        vec.put_u8(b'^');
        vec.put_u16(self.port);

        vec
    }
}