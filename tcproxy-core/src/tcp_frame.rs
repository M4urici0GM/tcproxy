use std::{
    fmt::Display,
    io::{Cursor, Read},
};

use bytes::{Buf, BufMut, BytesMut};

use crate::FrameError;

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

pub trait Frame {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameError> where Self : Sized;
    fn encode(&self) -> Vec<u8>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct ClientConnected;

#[derive(Debug, PartialEq, Eq)]
pub struct Ping;

#[derive(Debug, PartialEq, Eq)]
pub struct Pong;

#[derive(Debug, PartialEq, Eq)]
pub struct PortLimitReached;

#[derive(Debug, PartialEq, Eq)]
pub struct FailedToCreateProxy;

#[derive(Debug, PartialEq, Eq)]
pub struct ClientConnectedAck {
    port: u16
}

#[derive(Debug, PartialEq, Eq)]
pub struct IncomingSocket {
    connection_id: u32
}

#[derive(Debug, PartialEq, Eq)]
pub struct ClientUnableToConnect {
    connection_id: u32
}

#[derive(Debug, PartialEq, Eq)]
pub struct LocalConnectionDisconnected {
    connection_id: u32
}

#[derive(Debug, PartialEq, Eq)]
pub struct RemoteSocketDisconnected {
    connection_id: u32,
}

#[derive(Debug)]
pub enum TcpFrame {
    Ping(Ping),
    Pong(Pong),
    DataPacket(DataPacket),
    LocalConnectionDisconnected(LocalConnectionDisconnected),
    ClientUnableToConnect(ClientUnableToConnect),
    IncomingSocket(IncomingSocket),
    ClientConnectedAck(ClientConnectedAck),
    ClientConnected(ClientConnected),
    PortLimitReached(PortLimitReached),
    FailedToCreateProxy(FailedToCreateProxy),
    RemoteSocketDisconnected(RemoteSocketDisconnected),
}

impl LocalConnectionDisconnected {
    pub fn new(connection_id: &u32) -> Self {
        Self {
            connection_id: *connection_id,
        }
    }

    pub fn connection_id(&self) -> &u32 {
        &self.connection_id
    }
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

impl IncomingSocket {
    pub fn new(connection_id: &u32) -> Self {
        Self {
            connection_id: *connection_id,
        }
    }

    pub fn connection_id(&self) -> &u32 {
        &self.connection_id
    }
}

impl Frame for RemoteSocketDisconnected {
    fn decode(cursor: &mut Cursor<&[u8]>) -> Result<Self, FrameError> where Self : Sized {
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

impl Frame for DataPacket {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameError> where Self : Sized {
        let connection_id = get_u32(buffer)?;
        let buffer_size = get_u32(buffer)?;
        let buffer = seek_buffer(buffer, buffer_size)?;

        Ok(DataPacket::new(&connection_id, &buffer))
    }

    fn encode(&self) -> Vec<u8> {
        let mut final_buff = Vec::new();
        final_buff.put_u8(b'!');
        final_buff.put_u32(self.connection_id);
        final_buff.put_u32(self.buffer_size);
        final_buff.put_slice(&self.buffer[..]);

        final_buff
    }
}

impl Frame for ClientConnectedAck {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameError> where Self : Sized {
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


impl Frame for IncomingSocket {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameError> where Self : Sized {
        let connection_id = get_u32(buffer)?;
        Ok(Self { connection_id })
    }

    fn encode(&self) -> Vec<u8> {
        let mut final_buff = Vec::new();

        final_buff.put_u8(b'#');
        final_buff.put_u32(self.connection_id);

        final_buff
    }
}

impl ClientUnableToConnect {
    pub fn new(connection_id: &u32) -> Self {
        Self {
            connection_id: *connection_id,
        }
    }
}

impl Frame for ClientUnableToConnect {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameError> where Self: Sized {
        let connection_id = get_u32(buffer)?;
        Ok(Self { connection_id })
    }

    fn encode(&self) -> Vec<u8> {
        let mut final_buff = Vec::new();
        final_buff.put_u8(b'@');
        final_buff.put_u32(self.connection_id);

        final_buff
    }
}

impl Frame for LocalConnectionDisconnected {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameError> where Self: Sized {
        let connection_id = get_u32(buffer)?;
        Ok(Self { connection_id })
    }

    fn encode(&self) -> Vec<u8> {
        let mut final_buff = Vec::new();

        final_buff.put_u8(b'(');
        final_buff.put_u32(self.connection_id);

        final_buff
    }
}

impl TcpFrame {
    pub fn parse(cursor: &mut Cursor<&[u8]>) -> Result<TcpFrame, FrameError> {
        match get_u8(cursor)? {
            b'*' => Ok(TcpFrame::ClientConnected(ClientConnected)),
            b'-' => Ok(TcpFrame::Ping(Ping)),
            b'+' => Ok(TcpFrame::Pong(Pong)),
            b':' => Ok(TcpFrame::PortLimitReached(PortLimitReached)),
            b';' => Ok(TcpFrame::FailedToCreateProxy(FailedToCreateProxy)),
            b'^' => Ok(TcpFrame::ClientConnectedAck(ClientConnectedAck::decode(cursor)?)),
            b'$' => Ok(TcpFrame::RemoteSocketDisconnected(RemoteSocketDisconnected::decode(cursor)?)),
            b'#' => Ok(TcpFrame::IncomingSocket(IncomingSocket::decode(cursor)?)),
            b'@' => Ok(TcpFrame::ClientUnableToConnect(ClientUnableToConnect::decode(cursor)?)),
            b'(' => Ok(TcpFrame::LocalConnectionDisconnected(LocalConnectionDisconnected::decode(cursor)?)),
            b'!' => Ok(TcpFrame::DataPacket(DataPacket::decode(cursor)?)),
            actual => Err(format!("proto error. invalid frame type. {} {}", actual, String::from_utf8(vec![actual])?).into()),
        }
    }

    pub fn to_buffer(&self) -> BytesMut {
        let mut final_buff = BytesMut::new();

        match self {
            TcpFrame::ClientConnected(_) => {
                final_buff.put_u8(b'*');
            }
            TcpFrame::Ping(_) => {
                final_buff.put_u8(b'-');
            }
            TcpFrame::Pong(_) => {
                final_buff.put_u8(b'+');
            }
            TcpFrame::PortLimitReached(_) => {
                final_buff.put_u8(b':');
            },
            TcpFrame::FailedToCreateProxy(_) => {
                final_buff.put_u8(b';');
            },
            TcpFrame::ClientConnectedAck(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::RemoteSocketDisconnected(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::IncomingSocket(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::ClientUnableToConnect(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::LocalConnectionDisconnected(data) => final_buff.put_slice(&data.encode()),
            TcpFrame::DataPacket(data) => final_buff.put_slice(&data.encode()),
        };

        final_buff
    }
}

impl Display for TcpFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data_type = match self {
            TcpFrame::ClientConnected(_) => "ClientConnected".to_string(),
            TcpFrame::Ping(_) => "Ping".to_string(),
            TcpFrame::Pong(_) => "Pong".to_string(),
            TcpFrame::PortLimitReached(_) => "PortLimitReached".to_string(),
            TcpFrame::ClientConnectedAck(data) => format!("ClientConnectedACK ({})", data.port),
            TcpFrame::RemoteSocketDisconnected(data) => {
                format!("RemoteSocketDisconnected ({})", data.connection_id)
            }
            TcpFrame::IncomingSocket(data) => {
                format!("IncomingSocket ({})", data.connection_id)
            }
            TcpFrame::ClientUnableToConnect(data) => {
                format!("ClientUnableToConnect ({})", data.connection_id)
            }
            TcpFrame::DataPacket(data) => format!(
                "DataPacketHost, {}, size: {}, expected: {}",
                data.connection_id,
                data.buffer.len(),
                data.buffer_size
            ),
            TcpFrame::LocalConnectionDisconnected(data) => {
                format!("LocalClientDisconnected ({})", data.connection_id)
            },
            TcpFrame::FailedToCreateProxy(_) => "FailedToCreateProxy".to_string(),
        };

        let msg = format!("tcpframe: {}", data_type);
        write!(f, "{}", msg)
    }
}

fn check_cursor_size<T>(src: &mut Cursor<&[u8]>) -> Result<(), FrameError>
where
    T: Sized,
{
    if std::mem::size_of::<T>() > src.get_ref().len() - src.position() as usize {
        return Err(FrameError::Incomplete);
    }

    Ok(())
}

fn seek_buffer(src: &mut Cursor<&[u8]>, buffer_size: u32) -> Result<Vec<u8>, FrameError> {
    let mut buffer = vec![0; buffer_size as usize];
    src.read_exact(&mut buffer).map_err(|_| FrameError::Incomplete)?;
    Ok(buffer)
}

fn get_u32(src: &mut Cursor<&[u8]>) -> Result<u32, FrameError> {
    check_cursor_size::<u32>(src)?;
    Ok(src.get_u32())
}

fn get_u16(src: &mut Cursor<&[u8]>) -> Result<u16, FrameError> {
    check_cursor_size::<u16>(src)?;
    Ok(src.get_u16())
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    check_cursor_size::<u8>(src)?;
    Ok(src.get_u8())
}

#[cfg(test)]
mod tests {

}