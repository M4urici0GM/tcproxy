use std::fmt::Display;
use std::io::Cursor;
use bytes::{BufMut, BytesMut};

use crate::FrameDecodeError;
use crate::io::{get_u8};
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
        let frame = match get_u8(cursor)? {
            CLIENT_CONNECTED => TcpFrame::ClientConnected(ClientConnected::decode(cursor)?),
            CLIENT_CONNECTED_ACK => TcpFrame::ClientConnectedAck(ClientConnectedAck::decode(cursor)?),
            PING => TcpFrame::Ping(Ping::decode(cursor)?),
            PONG => TcpFrame::Pong(Pong::decode(cursor)?),
            SOCKET_CONNECTED => TcpFrame::SocketConnected(SocketConnected::decode(cursor)?),
            ERROR => TcpFrame::Error(Error::decode(cursor)?),
            DATA_PACKET_FRAME => TcpFrame::DataPacket(DataPacket::decode(cursor)?),
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



#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use bytes::BufMut;
    use chrono::Utc;
    use crate::{FrameDecodeError, is_type, TcpFrame};
    use crate::framing::ClientConnected;
    use crate::framing::frame_types::{PING, PONG};

    #[test]
    pub fn should_parse_client_connected() {
        // Arrange
        let buffer = vec![b'*'];
        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let frame = TcpFrame::parse(&mut cursor).unwrap();

        // Assert
        assert!(is_type!(frame, TcpFrame::ClientConnected(_)));
    }

    #[test]
    pub fn should_encode_client_connected() {
        // Arrange
        let frame = TcpFrame::ClientConnected(ClientConnected::new());

        // Act
        let result = TcpFrame::to_buffer(&frame);

        // Assert
        assert_eq!(&vec![b'*'], &result[..]);
    }

    #[test]
    pub fn should_parse_ping() {
        // Arrange
        let timestamp = Utc::now();
        let mut buffer = Vec::new();

        buffer.put_u8(PING);
        buffer.put_i64(timestamp.timestamp_millis());

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = TcpFrame::parse(&mut cursor).unwrap();

        // Assert
        assert!(is_type!(result, TcpFrame::Ping(_)));
    }

    #[test]
    pub fn parse_ping_should_return_err_if_buffer_is_missing_timestamp() {
        // Arrange
        let mut buffer = Vec::new();

        buffer.put_u8(PING);

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = TcpFrame::parse(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Incomplete));
    }

    #[test]
    pub fn parse_ping_should_return_err_if_timestamp_is_invalid() {
        // Arrange
        let mut buffer = Vec::new();

        buffer.put_u8(PING);
        buffer.put_i64(i64::MIN);

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = TcpFrame::parse(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Other(_)));
    }

    #[test]
    pub fn should_parse_pong() {
        // Arrange
        let timestamp = Utc::now();
        let mut buffer = Vec::new();

        buffer.put_u8(PONG);
        buffer.put_i64(timestamp.timestamp_millis());

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = TcpFrame::parse(&mut cursor).unwrap();

        // Assert
        assert!(is_type!(result, TcpFrame::Pong(_)));
    }

    #[test]
    pub fn parse_pong_should_return_err_if_buffer_is_missing_timestamp() {
        // Arrange
        let mut buffer = Vec::new();

        buffer.put_u8(PONG);

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = TcpFrame::parse(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Incomplete));
    }

    #[test]
    pub fn parse_pong_should_return_err_if_timestamp_is_invalid() {
        // Arrange
        let mut buffer = Vec::new();

        buffer.put_u8(PONG);
        buffer.put_i64(i64::MIN);

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = TcpFrame::parse(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Other(_)));
    }
}