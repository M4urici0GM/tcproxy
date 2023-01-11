mod data_packet;
mod client_connected_ack;
mod socket_disconnected;
mod client_connected;
mod socket_connected;
mod error;
mod ping;
mod pong;

pub use pong::*;
pub use ping::*;
pub use error::*;
pub use socket_disconnected::*;
pub use socket_connected::*;
pub use client_connected::*;
pub use client_connected_ack::*;
pub use data_packet::*;

pub mod frame_types {
    pub const PING: u8 = b'-';
    pub const PONG: u8 = b'+';
    pub const ERROR: u8 = b'@';
    pub const SOCKET_CONNECTED: u8 = b'#';
    pub const CLIENT_CONNECTED: u8 = b'*';
    pub const CLIENT_CONNECTED_ACK: u8 = b'^';
    pub const DATA_PACKET: u8 = b'!';
    pub const SOCKET_DISCONNECTED: u8 = b'(';
}

pub mod utils {
    use chrono::NaiveDateTime;
    use crate::FrameDecodeError;

    pub fn assert_connection_type(frame_type: &u8, expected_type: &u8) -> Result<(), FrameDecodeError> {
        if frame_type != expected_type {
            return Err(FrameDecodeError::UnexpectedFrameType(*frame_type));
        }

        Ok(())
    }

    pub fn parse_naive_date_time(timestamp_millis: &i64) -> Result<NaiveDateTime, FrameDecodeError> {
        match NaiveDateTime::from_timestamp_millis(*timestamp_millis) {
            Some(date) => Ok(date),
            None => {
                return Err(FrameDecodeError::Other(format!("failed to decode timestamp: {}", timestamp_millis).into()));
            }
        }
    }
}