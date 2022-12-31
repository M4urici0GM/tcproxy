use std::io::Cursor;
use crate::{Frame, FrameDecodeError};
use crate::io::get_u32;

#[derive(Debug, PartialEq, Eq)]
pub struct RemoteSocketDisconnected {
    connection_id: u32,
}

impl RemoteSocketDisconnected {
    pub fn new(connection_id: &u32) -> Self {
        Self {
            connection_id: *connection_id
        }
    }

    pub fn connection_id(&self) -> &u32 {
        &self.connection_id
    }
}

impl Frame for RemoteSocketDisconnected {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self : Sized {
        let connection_id = get_u32(cursor)?;
        Ok(Self { connection_id })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buff = Vec::new();
        buff.push(b'$');
        buff.extend_from_slice(&self.connection_id.to_be_bytes());

        buff
    }
}