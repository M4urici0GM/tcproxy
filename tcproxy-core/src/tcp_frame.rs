use std::fmt::Display;
use std::io::Cursor;
use bytes::{BufMut, BytesMut};

use crate::FrameDecodeError;
use crate::io::{get_u8};
use crate::framing::{
    ClientConnected,
    ClientConnectedAck,
    DataPacket,
    Error,
    IncomingSocket,
    LocalConnectionDisconnected,
    Ping,
    Pong,
    RemoteSocketDisconnected};


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
    LocalConnectionDisconnected(LocalConnectionDisconnected),
    IncomingSocket(IncomingSocket),
    ClientConnectedAck(ClientConnectedAck),
    ClientConnected(ClientConnected),
    RemoteSocketDisconnected(RemoteSocketDisconnected),
}

impl TcpFrame {
    pub fn parse(cursor: &mut Cursor<&[u8]>) -> Result<TcpFrame, FrameDecodeError> {
        match get_u8(cursor)? {
            b'*' => Ok(TcpFrame::ClientConnected(ClientConnected::decode(cursor)?)),
            b'-' => Ok(TcpFrame::Ping(Ping::decode(cursor)?)),
            b'+' => Ok(TcpFrame::Pong(Pong::decode(cursor)?)),
            b'^' => Ok(TcpFrame::ClientConnectedAck(ClientConnectedAck::decode(cursor)?)),
            b'$' => Ok(TcpFrame::RemoteSocketDisconnected(RemoteSocketDisconnected::decode(cursor)?)),
            b'#' => Ok(TcpFrame::IncomingSocket(IncomingSocket::decode(cursor)?)),
            b'@' => Ok(TcpFrame::Error(Error::decode(cursor)?)),
            b'(' => Ok(TcpFrame::LocalConnectionDisconnected(LocalConnectionDisconnected::decode(cursor)?)),
            b'!' => Ok(TcpFrame::DataPacket(DataPacket::decode(cursor)?)),
            actual => Err(format!("proto error. invalid frame type. {} {}", actual, String::from_utf8(vec![actual])?).into()),
        }
    }

    pub fn to_buffer(&self) -> BytesMut {
        let mut final_buff = BytesMut::new();

        match self {
            TcpFrame::ClientConnected(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::Ping(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::Pong(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::ClientConnectedAck(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::RemoteSocketDisconnected(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::IncomingSocket(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::Error(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::LocalConnectionDisconnected(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::DataPacket(data) => final_buff.put_slice(&data.encode()),
        };

        final_buff
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
            TcpFrame::RemoteSocketDisconnected(data) => {
                format!("RemoteSocketDisconnected ({})", data.connection_id())
            }
            TcpFrame::IncomingSocket(data) => {
                format!("IncomingSocket ({})", data.connection_id())
            }
            TcpFrame::DataPacket(data) => {
                format!("DataPacketHost, {}, size: {}", data.connection_id(), data.buffer().len())
            }
            TcpFrame::LocalConnectionDisconnected(data) => {
                format!("LocalClientDisconnected ({})", data.connection_id())
            },
            TcpFrame::Error(data) => {
                format!("Error[reason = {}]", data.reason())
            }
        };

        let msg = format!("tcpframe: {}", data_type);
        write!(f, "{}", msg)
    }
}

#[cfg(test)]
mod tests {}