use std::fmt;
use std::fmt::Formatter;
use std::io::Cursor;
use bytes::BufMut;
use tracing::trace;

use crate::{Frame, FrameDecodeError};
use crate::framing::frame_types::ERROR;
use crate::framing::utils::assert_connection_type;
use crate::io::{get_buffer, get_u16, get_u32};

use super::error_types::{CLIENT_UNABLE_TO_CONNECT, PORT_LIMIT_REACHED, FAILED_TO_CREATE_PROXY, AUTHENTICATION_FAILED, UNEXPECTED_ERROR, ALREADY_AUTHENTICATED};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Reason {
    ClientUnableToConnect,
    PortLimitReached,
    FailedToCreateProxy,
    AuthenticationFailed,
    AlreadyAuthenticated,
    UnexpectedError,
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
            Reason::ClientUnableToConnect => CLIENT_UNABLE_TO_CONNECT,
            Reason::PortLimitReached => PORT_LIMIT_REACHED,
            Reason::FailedToCreateProxy => FAILED_TO_CREATE_PROXY,
            Reason::AuthenticationFailed => AUTHENTICATION_FAILED,
            Reason::UnexpectedError => UNEXPECTED_ERROR,
            Reason::AlreadyAuthenticated => ALREADY_AUTHENTICATED,
        }
    }

    fn decode_reason(value: &u16) -> Result<Reason, FrameDecodeError> {
        match *value {
            CLIENT_UNABLE_TO_CONNECT => Ok(Reason::ClientUnableToConnect),
            PORT_LIMIT_REACHED => Ok(Reason::PortLimitReached),
            FAILED_TO_CREATE_PROXY => Ok(Reason::FailedToCreateProxy),
            AUTHENTICATION_FAILED => Ok(Reason::AuthenticationFailed),
            UNEXPECTED_ERROR => Ok(Reason::UnexpectedError),
            ALREADY_AUTHENTICATED => Ok(Reason::AlreadyAuthenticated),
            actual => {
                return Err(FrameDecodeError::Other(format!("invalid reason: {}", actual).into()));
            }
        }
    }
}

impl Frame for Error {
    fn decode(buffer: &mut Cursor<&[u8]>) -> Result<Self, FrameDecodeError> where Self: Sized {
        assert_connection_type(&get_u16(buffer)?, &ERROR)?;
        trace!("decoding Error frame");

        let value = get_u16(buffer)?;
        let reason = Error::decode_reason(&value)?;

        let data_len = get_u32(buffer)?;
        let data_buff = get_buffer(buffer, data_len)?;

        Ok(Self { reason, data: data_buff })
    }

    fn encode(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let reason = self.encode_reason();

        buffer.put_u16(ERROR);
        buffer.put_u16(reason);
        buffer.put_u32(0);
        buffer.put_slice(&vec![]);

        buffer
    }
}

impl fmt::Display for Reason {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Reason::AlreadyAuthenticated => format!("already authenticated!"),
            Reason::UnexpectedError => format!("encountered unexpected error!"),
            Reason::AuthenticationFailed => format!("authentication failed!"),
            Reason::FailedToCreateProxy => format!("Failed to create proxy"),
            Reason::PortLimitReached => format!("port limit reached"),
            Reason::ClientUnableToConnect => format!("target host unable to connect")
        };

        write!(f, "{}", format!("reason: {}", msg))
    }
}