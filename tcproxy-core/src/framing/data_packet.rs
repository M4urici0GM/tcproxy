use std::io::Cursor;
use bytes::BufMut;
use crate::FrameDecodeError;
use crate::framing::frame_types::DATA_PACKET_FRAME;
use crate::io::{get_buffer, get_u32};
use crate::tcp_frame::Frame;

#[derive(Debug, PartialEq, Eq)]
pub struct DataPacket {
    connection_id: u32,
    buffer_size: u32,
    buffer: Vec<u8>
}

impl DataPacket {
    pub fn new(connection_id: &u32, buffer: &[u8]) -> Self {
        Self {
            connection_id: *connection_id,
            buffer_size: buffer.len() as u32,
            buffer: buffer.to_owned(),
        }
    }

    pub fn connection_id(&self) -> &u32 {
        &self.connection_id
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}

impl Frame for DataPacket {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self : Sized {
        let connection_id = get_u32(buffer)?;
        let buffer_size = get_u32(buffer)?;
        let buffer = get_buffer(buffer, buffer_size)?;

        Ok(DataPacket::new(&connection_id, &buffer))
    }

    fn encode(&self) -> Vec<u8> {
        let mut final_buff = Vec::new();
        final_buff.put_u8(DATA_PACKET_FRAME);
        final_buff.put_u32(self.connection_id);
        final_buff.put_u32(self.buffer_size);
        final_buff.put_slice(&self.buffer[..]);

        final_buff
    }
}