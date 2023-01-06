use std::fmt;
use std::fmt::Formatter;
use std::io::Cursor;
use bytes::BufMut;
use crate::{Frame, FrameDecodeError};
use crate::framing::frame_types::ERROR;
use crate::framing::utils::assert_connection_type;
use crate::io::{get_buffer, get_u16, get_u32, get_u8};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Reason {
    ClientUnableToConnect,
    PortLimitReached,
    FailedToCreateProxy,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    reason: Reason,
    data: Vec<u8>,
}

impl Error {
    pub fn new(reason: &Reason, data: &[u8]) -> Self {
        Self {
            reason: reason.clone(),
            data: data.to_owned(),
        }
    }

    pub fn reason(&self) -> &Reason {
        &self.reason
    }

    fn encode_reason(&self) -> u16 {
        match &self.reason {
            Reason::ClientUnableToConnect => 0x10,
            Reason::PortLimitReached => 0x11,
            Reason::FailedToCreateProxy => 0x12,
        }
    }

    fn decode_reason(value: &u16) -> Result<Reason, FrameDecodeError> {
        match *value {
            0x10 => Ok(Reason::ClientUnableToConnect),
            0x11 => Ok(Reason::PortLimitReached),
            0x12 => Ok(Reason::FailedToCreateProxy),
            actual => {
                return Err(FrameDecodeError::Other(format!("invalid reason: {}", actual).into()));
            }
        }
    }
}

impl Frame for Error {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self: Sized {
        assert_connection_type(&get_u8(buffer)?, &ERROR)?;

        let value = get_u16(buffer)?;
        let reason = Error::decode_reason(&value)?;

        let data_len = get_u32(buffer)?;
        let data_buff = get_buffer(buffer, data_len)?;

        Ok(Self { reason, data: data_buff })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let reason = self.encode_reason();

        buffer.put_u8(ERROR);
        buffer.put_u16(reason);

        buffer
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Reason::FailedToCreateProxy => format!("Failed to create proxy"),
            Reason::PortLimitReached => format!("port limit reached"),
            Reason::ClientUnableToConnect => format!("target host unable to connect")
        };

        write!(f, "{}", format!("reason: {}", msg))
    }
}