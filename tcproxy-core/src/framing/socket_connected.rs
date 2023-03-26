use std::io::Cursor;
use bytes::BufMut;
use crate::{Frame, FrameDecodeError};
use crate::framing::frame_types::{PING, SOCKET_CONNECTED};
use crate::framing::utils::assert_connection_type;
use crate::io::{get_u32, get_u16};

#[derive(Debug, PartialEq, Eq)]
pub struct SocketConnected {
    connection_id: u32
}

impl SocketConnected {
    pub fn new(connection_id: &u32) -> Self {
        Self {
            connection_id: *connection_id,
        }
    }

    pub fn connection_id(&self) -> &u32 {
        &self.connection_id
    }
}


impl Frame for SocketConnected {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self : Sized {
        assert_connection_type(&get_u16(buffer)?, &SOCKET_CONNECTED)?;

        let connection_id = get_u32(buffer)?;
        Ok(Self { connection_id })
    }

    fn encode(&self) -> Vec<u8> {
        let mut final_buff = Vec::new();

        final_buff.put_u16(SOCKET_CONNECTED);
        final_buff.put_u32(self.connection_id);

        final_buff
    }
}