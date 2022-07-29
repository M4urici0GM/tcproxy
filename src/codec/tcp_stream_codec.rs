use std::io;
use std::io::ErrorKind;
use bytes::*;
use tokio_util::codec::{Decoder, Encoder};
use tracing::{debug, info};
use uuid::Uuid;

use crate::codec::TcpFrame;

pub struct TcpFrameCodec;

impl Encoder<TcpFrame> for TcpFrameCodec {
    type Error = io::Error;

    fn encode(&mut self, item: TcpFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            TcpFrame::ClientConnected => {
                dst.put_u8(1);
            }
            TcpFrame::IncomingSocket { connection_id } => {
                dst.put_u8(2);
                dst.put_u128(connection_id.as_u128());
            }
            TcpFrame::DataPacket { connection_id, buffer } => {
                dst.put_u8(3);
                dst.put_u128(connection_id.as_u128());
                dst.put_u32(buffer.len() as u32);
                dst.put_slice(&buffer[..]);
            }
        };

        Ok(())
    }
}

impl Decoder for TcpFrameCodec {
    type Item = TcpFrame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        info!("Received new DataPacket {}", src.len());

        let packet_type = src.get(0);
        let result = match packet_type.unwrap() {
            1 => {
                src.advance(1);
                Some(TcpFrame::ClientConnected)
            },
            2 if src.len() >= (16 + 1) => {
                let connection_id = Uuid::from_slice(&src[0..16]).unwrap();
                src.advance(1);
                Some(TcpFrame::IncomingSocket { connection_id })
            },
            3 if src.len() > (16 + 4 + 1) => {
                let buff = &src[17..21];
                let buffer_length = u32::from_be_bytes(buff.try_into().unwrap()) as usize;
                if (16 + 4 + buffer_length) > src.len() {
                    return Ok(None);
                }

                src.advance(1);
                let header = src.split_to(20);
                let connection_id = Uuid::from_slice(&header[0..16]).unwrap();
                let buffer = src.split_to(buffer_length);

                Some(TcpFrame::DataPacket { connection_id, buffer })
            }
            value => {
                debug!("Invalid Packet type received: {}. Closing Connection.", value);
                return Err(io::Error::new(ErrorKind::InvalidData, "Invalid data received!"));
            }
        };

        Ok(result)
    }
}