use std::io::Cursor;
use bytes::BufMut;
use chrono::{DateTime, Utc};
use crate::{Frame, FrameDecodeError};
use crate::framing::frame_types::PING;
use crate::framing::utils::{assert_connection_type, parse_naive_date_time};
use crate::io::{get_i64, get_u16};


#[derive(Debug, PartialEq, Eq)]
pub struct Ping {
    timestamp: DateTime<Utc>
}

impl Ping {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
        }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
}

impl Frame for Ping {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Ping, FrameDecodeError> where Self: Sized {
        assert_connection_type(&get_u16(buffer)?, &PING)?;

        let timestamp_millis = get_i64(buffer)?;
        let naive_datetime = parse_naive_date_time(&timestamp_millis)?;

        return Ok(Self {
            timestamp: DateTime::from_utc(naive_datetime, Utc),
        })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.put_u16(PING);
        buffer.put_i64(self.timestamp.timestamp_millis());

        buffer
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use bytes::BufMut;
    use chrono::Utc;
    use crate::tcp_frame::Frame;
    use crate::framing::Ping;
    use crate::{FrameDecodeError, is_type};
    use crate::framing::frame_types::PING;

    #[test]
    pub fn should_parse_ping() {
        // Arrange
        let timestamp = Utc::now();
        let mut buffer = Vec::new();

        buffer.put_u16(PING);
        buffer.put_i64(timestamp.timestamp_millis());

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Ping::decode(&mut cursor).unwrap();

        // Assert
        assert_eq!(
            timestamp.timestamp_millis(),
            result.timestamp().timestamp_millis());
    }

    #[test]
    pub fn parse_ping_should_return_err_if_buffer_is_missing_timestamp() {

        // Arrange
        let buffer: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Ping::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Incomplete));
    }

    #[test]
    pub fn parse_ping_should_return_err_if_timestamp_is_invalid() {
        // Arrange
        let mut buffer = Vec::new();
        buffer.put_u16(PING);
        buffer.put_i64(i64::MIN);

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Ping::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Other(_)));
    }
}