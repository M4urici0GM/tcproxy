use std::io::Cursor;
use chrono::{DateTime, Utc};
use crate::{Frame, FrameDecodeError};

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
    fn decode(_: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self: Sized {
        return Ok(Self::new())
    }

    fn encode(&self) -> Vec<u8> {
        return vec![b'-'];
    }
}