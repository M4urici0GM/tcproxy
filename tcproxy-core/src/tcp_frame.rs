use bytes::{Buf, BytesMut};
use std::fmt::Display;
use std::io::Cursor;
use tracing::debug;

use crate::framing::frame_types::*;
use crate::framing::*;
use crate::FrameDecodeError;

pub trait Frame {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError>
    where
        Self: Sized;
    fn encode(&self) -> Vec<u8>;
}

#[derive(Debug, Clone)]
pub enum TcpFrame {
    Ping(Ping),
    Pong(Pong),
    Error(Error),
    Authenticate(Authenticate),
    AuthenticateAck(AuthenticateAck),
    DataPacket(DataPacket),
    SocketConnected(SocketConnected),
    ClientConnectedAck(ClientConnectedAck),
    ClientConnected(ClientConnected),
    SocketDisconnected(SocketDisconnected),
}

impl TcpFrame {
    pub fn parse(cursor: &mut Cursor<&[u8]>) -> Result<TcpFrame, FrameDecodeError> {
        if !cursor.has_remaining() {
            return Err(FrameDecodeError::Incomplete);
        }
        let grant_type_buf = [cursor.chunk()[0], cursor.chunk()[1]];
        let raw_grant_type = u16::from_be_bytes(grant_type_buf);

        let frame = match raw_grant_type {
            CLIENT_CONNECTED => TcpFrame::ClientConnected(ClientConnected::decode(cursor)?),
            CLIENT_CONNECTED_ACK => {
                TcpFrame::ClientConnectedAck(ClientConnectedAck::decode(cursor)?)
            }
            PING => TcpFrame::Ping(Ping::decode(cursor)?),
            PONG => TcpFrame::Pong(Pong::decode(cursor)?),
            SOCKET_CONNECTED => TcpFrame::SocketConnected(SocketConnected::decode(cursor)?),
            ERROR => TcpFrame::Error(Error::decode(cursor)?),
            DATA_PACKET => TcpFrame::DataPacket(DataPacket::decode(cursor)?),
            AUTHENTICATE => TcpFrame::Authenticate(Authenticate::decode(cursor)?),
            AUTHENTICATE_ACK => TcpFrame::AuthenticateAck(AuthenticateAck::decode(cursor)?),
            actual => return Err(format!("proto error. invalid frame type. {}", actual).into()),
        };

        Ok(frame)
    }

    pub fn to_buffer(&self) -> BytesMut {
        debug!("sending frame {}", &self);
        let buffer = match self {
            TcpFrame::AuthenticateAck(data) => data.encode(),
            TcpFrame::Authenticate(data) => data.encode(),
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
            TcpFrame::ClientConnected(_) => "ClientConnected".to_string(),
            TcpFrame::Ping(_) => "Ping".to_string(),
            TcpFrame::Pong(_) => "Pong".to_string(),
            TcpFrame::Authenticate(_) => {
                "Authenticate".to_string()
            }
            TcpFrame::AuthenticateAck(_) => {
                "AuthenticateAck".to_string()
            }
            TcpFrame::ClientConnectedAck(_) => {
                "ClientConnectedACK".to_string()
            }
            TcpFrame::SocketConnected(data) => {
                format!("IncomingSocket ({})", data.connection_id())
            }
            TcpFrame::SocketDisconnected(data) => {
                format!("Socket Disconnected ({})", data.connection_id())
            }
            TcpFrame::DataPacket(data) => {
                format!(
                    "DataPacketHost, {}, size: {}",
                    data.connection_id(),
                    data.buffer().len()
                )
            }
            TcpFrame::Error(data) => {
                format!("Error[reason = {}]", data.reason())
            }
        };

        let msg = format!("tcpframe: {}", data_type);
        write!(f, "{}", msg)
    }
}

impl From<Authenticate> for TcpFrame {
    fn from(value: Authenticate) -> Self {
        Self::Authenticate(value)
    }
}
