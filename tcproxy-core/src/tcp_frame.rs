use std::fmt::Display;
use std::io::Cursor;
use bytes::{Buf, BytesMut};

use crate::FrameDecodeError;

use crate::framing::frame_types::*;
use crate::framing::{ClientConnected, ClientConnectedAck, DataPacket, Error, SocketConnected, Ping, Pong, SocketDisconnected};


pub trait Frame {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self: Sized;
    fn encode(&self) -> Vec<u8>;
}

#[derive(Debug)]
pub enum TcpFrame {
    Ping(Ping),
    Pong(Pong),
    Error(Error),
    DataPacket(DataPacket),
    SocketConnected(SocketConnected),
    ClientConnectedAck(ClientConnectedAck),
    ClientConnected(ClientConnected),
    SocketDisconnected(SocketDisconnected)
}

impl TcpFrame {
    pub fn parse(cursor: &mut Cursor<&[u8]>) -> Result<TcpFrame, FrameDecodeError> {
        if !cursor.has_remaining() {
            return Err(FrameDecodeError::Incomplete);
        }

        let frame = match cursor.chunk()[0] {
            CLIENT_CONNECTED => TcpFrame::ClientConnected(ClientConnected::decode(cursor)?),
            CLIENT_CONNECTED_ACK => TcpFrame::ClientConnectedAck(ClientConnectedAck::decode(cursor)?),
            PING => TcpFrame::Ping(Ping::decode(cursor)?),
            PONG => TcpFrame::Pong(Pong::decode(cursor)?),
            SOCKET_CONNECTED => TcpFrame::SocketConnected(SocketConnected::decode(cursor)?),
            ERROR => TcpFrame::Error(Error::decode(cursor)?),
            DATA_PACKET => TcpFrame::DataPacket(DataPacket::decode(cursor)?),
            actual => {
                let msg = format!(
                    "proto error. invalid frame type. {} {}",
                    actual,
                    String::from_utf8(vec![actual])?);

                return Err(msg.into())
            },
        };

        Ok(frame)
    }

    pub fn to_buffer(&self) -> BytesMut {
        let buffer = match self {
            TcpFrame::ClientConnected(data) => data.encode(),
            TcpFrame::ClientConnectedAck(data) => data.encode(),
            TcpFrame::Ping(data) => data.encode(),
            TcpFrame::Pong(data) => data.encode(),
            TcpFrame::SocketConnected(data) => data.encode(),
            TcpFrame::SocketDisconnected(data) => data.encode(),
            TcpFrame::Error(data) => data.encode(),
            TcpFrame::DataPacket(data) => data.encode(),
        };

        BytesMut::from(&buffer[..])
    }
}

impl Display for TcpFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data_type = match self {
            TcpFrame::ClientConnected(_) => {
                "ClientConnected".to_string()
            }
            TcpFrame::Ping(_) => {
                "Ping".to_string()
            }
            TcpFrame::Pong(_) => {
                "Pong".to_string()
            }
            TcpFrame::ClientConnectedAck(data) => {
                format!("ClientConnectedACK ({})", data.port())
            }
            TcpFrame::SocketConnected(data) => {
                format!("IncomingSocket ({})", data.connection_id())
            }
            TcpFrame::SocketDisconnected(data) => {
                format!("Socket Disconnected ({})", data.connection_id())
            }
            TcpFrame::DataPacket(data) => {
                format!("DataPacketHost, {}, size: {}", data.connection_id(), data.buffer().len())
            }
            TcpFrame::Error(data) => {
                format!("Error[reason = {}]", data.reason())
            }
        };

        let msg = format!("tcpframe: {}", data_type);
        write!(f, "{}", msg)
    }
}
