mod authenticate;
mod authenticate_ack;
mod client_connected;
mod client_connected_ack;
mod data_packet;
mod error;
mod ping;
mod pong;
mod socket_connected;
mod socket_disconnected;

pub use authenticate::*;
pub use authenticate_ack::*;
pub use client_connected::*;
pub use client_connected_ack::*;
pub use data_packet::*;
pub use error::*;
pub use ping::*;
pub use pong::*;
pub use socket_connected::*;
pub use socket_disconnected::*;

pub mod frame_types {
    pub const PING: u16 = 0x15;
    pub const PONG: u16 = 0x16;
    pub const ERROR: u16 = 0x97;
    pub const SOCKET_CONNECTED: u16 = 0x18;
    pub const CLIENT_CONNECTED: u16 = 0x19;
    pub const CLIENT_CONNECTED_ACK: u16 = 0x20;
    pub const DATA_PACKET: u16 = 0x21;
    pub const SOCKET_DISCONNECTED: u16 = 0x22;
    pub const AUTHENTICATE: u16 = 0x23;
    pub const AUTHENTICATE_ACK: u16 = 0x24;
    pub const LOGIN: u16 = 0x25;
}

pub mod error_types {
    pub const CLIENT_UNABLE_TO_CONNECT: u16 = 0x99;
    pub const PORT_LIMIT_REACHED: u16 = 0x98;
    pub const FAILED_TO_CREATE_PROXY: u16 = 0x97;
    pub const AUTHENTICATION_FAILED: u16 = 0x96;
    pub const UNEXPECTED_ERROR: u16 = 0x95;
    pub const ALREADY_AUTHENTICATED: u16 = 0x94;
}

pub mod authentication_grant_types {
    pub const PASSWORD_AUTHENTICATION: u16 = 0x10;
    pub const AUTH_TOKEN_AUTHENTICATION: u16 = 0x11;
}

pub mod utils {
    use crate::FrameDecodeError;
    use chrono::NaiveDateTime;

    pub fn assert_connection_type(
        frame_type: &u16,
        expected_type: &u16,
    ) -> Result<(), FrameDecodeError> {
        if frame_type != expected_type {
            return Err(FrameDecodeError::UnexpectedFrameType(*frame_type));
        }

        Ok(())
    }

    pub fn parse_naive_date_time(
        timestamp_millis: &i64,
    ) -> Result<NaiveDateTime, FrameDecodeError> {
        match NaiveDateTime::from_timestamp_millis(*timestamp_millis) {
            Some(date) => Ok(date),
            None => {
                return Err(FrameDecodeError::Other(
                    format!("failed to decode timestamp: {}", timestamp_millis).into(),
                ));
            }
        }
    }
}
