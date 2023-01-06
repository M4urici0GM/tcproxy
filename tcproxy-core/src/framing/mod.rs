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
    pub const DATA_PACKET_FRAME: u8 = b'!';
    pub const SOCKET_DISCONNECTED: u8 = b'(';
}