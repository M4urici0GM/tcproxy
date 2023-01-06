use std::io::Cursor;
use crate::{Frame, FrameDecodeError};
use crate::framing::frame_types::SOCKET_DISCONNECTED;
use crate::io::get_u32;

#[derive(Debug, PartialEq, Eq)]
pub struct SocketDisconnected {
    connection_id: u32,
}

impl SocketDisconnected {
    pub fn new(connection_id: &u32) -> Self {
        Self {
            connection_id: *connection_id
        }
    }

    pub fn connection_id(&self) -> &u32 {
        &self.connection_id
    }
}

impl Frame for SocketDisconnected {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self : Sized {
        let connection_id = get_u32(cursor)?;
        Ok(Self { connection_id })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buff = Vec::new();
        buff.push(SOCKET_DISCONNECTED);
        buff.extend_from_slice(&self.connection_id.to_be_bytes());

        buff
    }
}