use std::{io};
use std::io::{ErrorKind, Cursor, Read};
use bytes::*;
use tokio_util::codec::{Decoder, Encoder};
use tracing::{debug, info};
use uuid::Uuid;

use crate::codec::TcpFrame;

pub struct TcpFrameCodec {
    pub source: String,
}

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
            TcpFrame::DataPacketClient { connection_id, buffer } => {
                dst.put_u8(3);
                dst.put_u128(connection_id.as_u128());
                dst.put_u32(buffer.len() as u32);
                dst.put_slice(&buffer[..]);
            },
            TcpFrame::DataPacketHost { connection_id, buffer } => {
                dst.put_u8(10);
                dst.put_u128(connection_id.as_u128());
                dst.put_u32(buffer.len() as u32);
                dst.put_slice(&buffer[..]);
            },
            TcpFrame::ClientConnectedAck { port } => {
                dst.put_u8(4);
                dst.put_u16(port);
            },
            TcpFrame::ClientUnableToConnect { connection_id } => {
                dst.put_u8(5);
                dst.put_u128(connection_id.as_u128());
            },
            TcpFrame::LocalClientDisconnected { connection_id } => {
                dst.put_u8(6);
                dst.put_u128(connection_id.as_u128());
            },
            TcpFrame::Ping => {
                dst.put_u8(7);
            },
            TcpFrame::Pong => {
                dst.put_u8(8);
            },
            TcpFrame::RemoteSocketDisconnected { connection_id } => {
                dst.put_u8(9);
                dst.put_u128(connection_id.as_u128());
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

        let mut cursor = Cursor::new(&src[..]);
        let packet_type = cursor.get_u8();
        let remaining_size = src.len() as usize - cursor.position() as usize;
        let result = match packet_type {
            1 => Some(TcpFrame::ClientConnected),
            2 if remaining_size >= (16) => {
                let connection_id_buf = cursor.get_u128();
                let connection_id = Uuid::from_u128(connection_id_buf);
                Some(TcpFrame::IncomingSocket { connection_id })
            },
            3 if remaining_size >= (16 + 4) => {
                let connection_id_buf = cursor.get_u128();
                let buffer_size = cursor.get_u32() as usize;

                if buffer_size > src.len() as usize - cursor.position() as usize {
                    debug!("received less bytes than expected {}", buffer_size);
                    return Ok(None);
                }

                let mut buffer = vec![0u8; buffer_size as usize];
                let connection_id = Uuid::from_u128(connection_id_buf);

                cursor.read_exact(&mut buffer)?;
                Some(TcpFrame::DataPacketClient { connection_id, buffer: BytesMut::from(&buffer[..]) })
            },
            10 if remaining_size >= (16 + 4) => {
                let connection_id_buf = cursor.get_u128();
                let buffer_size = cursor.get_u32() as usize;

                if buffer_size > src.len() as usize - cursor.position() as usize {
                    debug!("received less bytes than expected");
                    return Ok(None);
                }

                let mut buffer = vec![0u8; buffer_size as usize];
                let connection_id = Uuid::from_u128(connection_id_buf);

                cursor.read_exact(&mut buffer)?;
                Some(TcpFrame::DataPacketHost { connection_id, buffer: BytesMut::from(&buffer[..]) })
            },
            4 if remaining_size >= 2 => {
                let port = cursor.get_u16();

                Some(TcpFrame::ClientConnectedAck { port })
            },
            5 if remaining_size >= 16 => {
                let connection_id_buf = cursor.get_u128();
                let connection_id = Uuid::from_u128(connection_id_buf);

                Some(TcpFrame::LocalClientDisconnected { connection_id })
            },
            6 if remaining_size >= 16 => {
                let connection_id_buf = cursor.get_u128();
                let connection_id = Uuid::from_u128(connection_id_buf);

                Some(TcpFrame::LocalClientDisconnected { connection_id })
            },
            7 => Some(TcpFrame::Ping),
            8 => Some(TcpFrame::Pong),
            9 if remaining_size >= 16 => {
                let connection_id_buf = cursor.get_u128();
                let connection_id = Uuid::from_u128(connection_id_buf);

                Some(TcpFrame::RemoteSocketDisconnected { connection_id })
            },
            value => {
                debug!("looks like data is wrong [{}] {:?}", src.len(), src.to_vec());

                None
            },
        };


        src.advance(cursor.position() as usize);
        Ok(result)
    }
}