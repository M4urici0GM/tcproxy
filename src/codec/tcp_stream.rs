use std::{fmt::Display, io::{Cursor, Read}, string::FromUtf8Error, num::TryFromIntError};

use bytes::{BytesMut, Buf, BufMut};
use tracing::trace;
use uuid::Uuid;

pub enum TcpFrame {
    ClientConnected,
    Ping,
    Pong,
    ClientConnectedAck {
        port: u16,
    },
    RemoteSocketDisconnected {
        connection_id: Uuid
    },
    IncomingSocket {
        connection_id: Uuid
    },
    ClientUnableToConnect {
        connection_id: Uuid
    },
    LocalClientDisconnected {
        connection_id: Uuid
    },
    DataPacketClient {
        connection_id: Uuid,
        buffer: BytesMut,
    },
    DataPacketHost {
        connection_id: Uuid,
        buffer: BytesMut,
    },
    PortLimitReached,
}

#[derive(Debug)]
pub enum FrameError {
    Incomplete,
    Other(crate::Error),
}

impl TcpFrame {
    pub fn check(cursor: &mut Cursor<&[u8]>) -> Result<(), FrameError> {
        trace!("checking if buffer has available frame.. [length: {}]", cursor.get_ref().len());
        match get_u8(cursor)? {
            b'*' => Ok(()),
            b'-' => Ok(()),
            b'+' => Ok(()),
            b':' => Ok(()),
            b'^' => {
                let _ = get_u16(cursor)?;
                Ok(())
            },
            b'$' => {
                let _ = get_u128(cursor)?;
                Ok(())
            },
            b'#' => {
                let _ = get_u128(cursor)?;
                Ok(())
            },
            b'@' => {
                let _ = get_u128(cursor)?;
                Ok(())
            },
            b'(' => {
                let _ = get_u128(cursor)?;
                Ok(())
            },
            b')' => {
                let _ = get_u128(cursor)?;
                let size = get_u32(cursor)?;
                let _ = seek_buffer(cursor, size)?;

                Ok(())
            },
            b'!' => {
                let _ = get_u128(cursor)?;
                let size = get_u32(cursor)?;
                let _ = seek_buffer(cursor, size)?;

                Ok(())
            },
            actual => Err(format!("proto error. invalid frame type. {}", actual).into()),
        }
    }

    pub fn parse(cursor: &mut Cursor<&[u8]>) -> Result<TcpFrame, FrameError> {
        match get_u8(cursor)? {
            b'*' => Ok(TcpFrame::ClientConnected),
            b'-' => Ok(TcpFrame::Ping),
            b'+' => Ok(TcpFrame::Pong),
            b'^' => {
                let port = get_u16(cursor)?;
                Ok(TcpFrame::ClientConnectedAck { port })
            },
            b'$' => {
                let value = get_u128(cursor)?;
                let connection_id = Uuid::from_u128(value);
                Ok(TcpFrame::RemoteSocketDisconnected { connection_id })
            },
            b'#' => {
                let value = get_u128(cursor)?;
                let connection_id = Uuid::from_u128(value);
                Ok(TcpFrame::IncomingSocket { connection_id })
            },
            b'@' => {
                let value = get_u128(cursor)?;
                let connection_id = Uuid::from_u128(value);
                Ok(TcpFrame::ClientUnableToConnect { connection_id })
            },
            b'(' => {
                let value = get_u128(cursor)?;
                let connection_id = Uuid::from_u128(value);
                Ok(TcpFrame::LocalClientDisconnected { connection_id })
            },
            b')' => {
                let connection_id_value = get_u128(cursor)?;
                let buffer_size = get_u32(cursor)?;
                let buffer = seek_buffer(cursor, buffer_size)?;

                let connection_id = Uuid::from_u128(connection_id_value);
                Ok(TcpFrame::DataPacketClient { connection_id, buffer })
            },
            b':' => {
                trace!("found DataPacketHost frame, buffer size: {}, cursor_pos: {}", cursor.get_ref().len(), cursor.position());
                let connection_id_value = get_u128(cursor)?;
                let buff_size = get_u32(cursor)?;
                let buffer = seek_buffer(cursor, buff_size)?;

                trace!("supposed buffer size: {}, actual buffer size: {}", buff_size, buffer.len());

                let connection_id = Uuid::from_u128(connection_id_value);
                Ok(TcpFrame::DataPacketHost { connection_id, buffer })
            },
            actual => Err(format!("proto error. invalid frame type. {}", actual).into()),
        }
    }

    pub fn to_buffer(&self) -> BytesMut {
        let mut final_buff = BytesMut::new();

        match self {
            TcpFrame::ClientConnected => {
                final_buff.put_u8(b'*');
            },
            TcpFrame::Ping => {
                final_buff.put_u8(b'-');
            },
            TcpFrame::Pong => {
                final_buff.put_u8(b'+');
            },
            TcpFrame::ClientConnectedAck { port } => {
                final_buff.put_u8(b'^');
                final_buff.put_u16(*port);
            }
            TcpFrame::RemoteSocketDisconnected { connection_id } => {
                final_buff.put_u8(b'$');
                final_buff.put_u128(connection_id.as_u128());
            },
            TcpFrame::IncomingSocket { connection_id } => {
                final_buff.put_u8(b'#');
                final_buff.put_u128(connection_id.as_u128());
            },
            TcpFrame::ClientUnableToConnect { connection_id } => {
                final_buff.put_u8(b'@');
                final_buff.put_u128(connection_id.as_u128());
            },
            TcpFrame::LocalClientDisconnected { connection_id } => {
                final_buff.put_u8(b'(');
                final_buff.put_u128(connection_id.as_u128());
            },
            TcpFrame::DataPacketClient { connection_id, buffer } => {
                final_buff.put_u8(b')');
                final_buff.put_u128(connection_id.as_u128());
                final_buff.put_u32(buffer.len() as u32);
                final_buff.put_slice(&buffer[..]);
            },
            TcpFrame::DataPacketHost { connection_id, buffer } => {
                final_buff.put_u8(b'!');
                final_buff.put_u128(connection_id.as_u128());
                final_buff.put_u32(buffer.len() as u32);
                final_buff.put_slice(&buffer[..]);
            },
            TcpFrame::PortLimitReached => {
                final_buff.put_u8(b'^');
            }
        };

        final_buff
    }
}

impl Display for TcpFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data_type = match self {
            TcpFrame::ClientConnected => "ClientConnected".to_string(),
            TcpFrame::Ping => format!("Ping"),
            TcpFrame::Pong => format!("Pong"),
            TcpFrame::PortLimitReached => format!("PortLimitReached"),
            TcpFrame::ClientConnectedAck { port } => format!("ClientConnectedACK ({})", port),
            TcpFrame::RemoteSocketDisconnected { connection_id } => format!("RemoteSocketDisconnected ({})", connection_id),
            TcpFrame::IncomingSocket { connection_id } => format!("IncomingSocket ({})", connection_id),
            TcpFrame::ClientUnableToConnect { connection_id } => format!("ClientUnableToConnect ({})", connection_id),
            TcpFrame::DataPacketClient { connection_id, buffer } => format!("DataPacketClient, {}, size: {}", connection_id, buffer.len()),
            TcpFrame::DataPacketHost { connection_id, buffer } => format!("DataPacketHost, {}, size: {}", connection_id, buffer.len()),
            TcpFrame::LocalClientDisconnected { connection_id } => format!("LocalClientDisconnected ({})", connection_id),
        };

        let msg = format!("tcpframe: {}", data_type);
        write!(f, "{}", msg)
    }
}

impl From<String> for FrameError {
    fn from(src: String) -> FrameError {
        FrameError::Other(src.into())
    }
}

impl From<&str> for FrameError {
    fn from(src: &str) -> FrameError {
        FrameError::Other(src.into())
    }
}

impl std::error::Error for FrameError {}

impl From<FromUtf8Error> for FrameError {
    fn from(_src: FromUtf8Error) -> FrameError {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for FrameError {
    fn from(_src: TryFromIntError) -> FrameError {
        "protocol error; invalid frame format".into()
    }
}


impl std::fmt::Display for FrameError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FrameError::Incomplete => "stream ended early".fmt(fmt),
            FrameError::Other(err) => err.fmt(fmt),
        }
    }
}

fn check_cursor_size<T>(src: &mut Cursor<&[u8]>) -> Result<(), FrameError> where T : Sized {
    if std::mem::size_of::<T>() > src.get_ref().len() - src.position() as usize {
        return Err(FrameError::Incomplete);
    }

    Ok(())
}

fn seek_buffer(src: &mut Cursor<&[u8]>, buffer_size: u32) -> Result<BytesMut, FrameError> {
    let start = src.position() as usize;
    if buffer_size as usize > src.get_ref().len() - start {
        return Err(FrameError::Incomplete);
    }
    

    let mut buffer = vec![0u8; buffer_size as usize];
    match src.read_exact(&mut buffer) {
        Ok(_) => {},
        Err(err) => return Err(FrameError::Other(err.into())),
    };

    Ok(BytesMut::from(&buffer[..]))
}

fn get_u32(src: &mut Cursor<&[u8]>) -> Result<u32, FrameError> {
    check_cursor_size::<u32>(src)?;
    Ok(src.get_u32())
}

fn get_u128(src: &mut Cursor<&[u8]>) -> Result<u128, FrameError> {
    check_cursor_size::<u128>(src)?;
    Ok(src.get_u128())
}

fn get_u16(src: &mut Cursor<&[u8]>) -> Result<u16, FrameError> {
    check_cursor_size::<u16>(src)?;
    Ok(src.get_u16())
}

fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    check_cursor_size::<u8>(src)?;
    Ok(src.get_u8())
}