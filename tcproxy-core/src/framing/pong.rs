use crate::{Frame, FrameDecodeError};
use bytes::BufMut;
use std::io::Cursor;

use crate::framing::frame_types::PONG;
use crate::framing::utils::{assert_connection_type, parse_naive_date_time};
use crate::io::{get_i64, get_u16};
use chrono::{DateTime, Utc};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pong {
    timestamp: DateTime<Utc>,
}

impl Pong {
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
        }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> {
        &self.timestamp
    }
}

impl Frame for Pong {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError>
    where
        Self: Sized,
    {
        assert_connection_type(&get_u16(buffer)?, &PONG)?;

        let timestamp_millis = get_i64(buffer)?;
        let naive_datetime = parse_naive_date_time(&timestamp_millis)?;

        return Ok(Self {
            timestamp: DateTime::from_utc(naive_datetime, Utc),
        });
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.put_u16(PONG);
        buffer.put_i64(self.timestamp.timestamp_millis());

        return buffer;
    }
}

#[cfg(test)]
mod tests {
    use crate::framing::frame_types::PONG;
    use crate::framing::Pong;
    use crate::tcp_frame::Frame;
    use crate::{is_type, FrameDecodeError};
    use bytes::BufMut;
    use chrono::Utc;
    use rand::random;
    use std::io::Cursor;

    #[test]
    pub fn should_parse_pong() {
        // Arrange
        let timestamp = Utc::now();
        let mut buffer = Vec::new();

        buffer.put_u16(PONG);
        buffer.put_i64(timestamp.timestamp_millis());

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Pong::decode(&mut cursor).unwrap();

        // Assert
        assert_eq!(
            timestamp.timestamp_millis(),
            result.timestamp.timestamp_millis()
        );
    }

    #[test]
    pub fn parse_pong_should_return_err_if_buffer_is_missing_timestamp() {
        // Arrange
        let mut buffer = Vec::new();

        buffer.put_u16(PONG);

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Pong::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Incomplete));
    }

    #[test]
    pub fn parse_pong_should_return_err_if_timestamp_is_invalid() {
        // Arrange
        let mut buffer = Vec::new();

        buffer.put_u16(PONG);
        buffer.put_i64(i64::MIN);

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Pong::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), FrameDecodeError::Other(_)));
    }

    #[test]
    pub fn parse_pong_should_return_err_when_unexpected_frame_type() {
        // Arrange
        let mut buffer: Vec<u8> = vec![];
        buffer.put_u16(random());

        let mut cursor = Cursor::new(&buffer[..]);

        // Act
        let result = Pong::decode(&mut cursor);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(
            result.unwrap_err(),
            FrameDecodeError::UnexpectedFrameType(_)
        ))
    }
}
