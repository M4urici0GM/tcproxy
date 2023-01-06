use std::io::Cursor;
use bytes::BufMut;
use chrono::{DateTime, NaiveDateTime, Utc};
use crate::{Frame, FrameDecodeError};
use crate::framing::frame_types::PING;
use crate::io::get_i64;


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
        let timestamp_millis = get_i64(buffer)?;
        let naive_datetime = match NaiveDateTime::from_timestamp_millis(timestamp_millis) {
            Some(date) => date,
            None => {
                return Err(FrameDecodeError::Other(format!("failed to decode timestamp: {}", timestamp_millis).into()));
            }
        };

        return Ok(Self {
            timestamp: DateTime::from_utc(naive_datetime, Utc),
        })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.put_u8(PING);
        buffer.put_i64(self.timestamp.timestamp_millis());

        buffer
    }
}