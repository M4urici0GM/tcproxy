mod command;
mod frame_error;
mod tcp_frame;

pub mod tcp;
pub mod transport;
pub mod framing;
pub mod io;
pub mod config;
pub mod auth;
pub mod http;

use std::io::{Cursor, Read};
use bytes::{BufMut, Buf};
use mongodb::bson::Uuid;

pub use command::*;
pub use frame_error::*;
pub use tcp_frame::*;

pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

pub trait PutU32String: BufMut {
    fn put_u32_sized_str(&mut self, value: &str);
}

pub trait ReadU32String: Read {
    fn read_u32_str(&mut self) -> std::io::Result<String>;
}

pub trait PutBsonUuid: BufMut {
    fn put_bson_uuid(&mut self, value: &Uuid);
}

pub trait ReadBsonUuid: Read {
    fn read_bson_uuid(&mut self) -> std::io::Result<Uuid>;
}

impl ReadBsonUuid for Cursor<&[u8]> {
    /// Reads and consumes 16 bytes from Cursor, Converting it to an UUID instance.
    fn read_bson_uuid(&mut self) -> std::io::Result<Uuid> {
        let mut buffer: [u8; 16] = Default::default();
        self.read_exact(&mut buffer)?;

        Ok(Uuid::from_bytes(buffer))
    }
}

impl ReadU32String for Cursor<&[u8]> {
    fn read_u32_str(&mut self) -> std::io::Result<String> {
        let buffer_size = self.get_u32();
        let mut buffer = vec![0; buffer_size as usize];

        self.read_exact(&mut buffer)?;

        String::from_utf8(buffer).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Unsupported, "invalid utf-8 string")
        })
    }
}

impl PutBsonUuid for Vec<u8> {
    fn put_bson_uuid(&mut self, value: &Uuid) {
        self.put_slice(&value.bytes());
    }
}

impl PutU32String for Vec<u8> {
    fn put_u32_sized_str(&mut self, value: &str) {
        self.put_u32(value.len() as u32);
        self.put_slice(value.as_bytes());
    }
}

pub mod test_util {

    pub fn generate_random_buffer(buffer_size: i32) -> bytes::BytesMut {
        use bytes::{BufMut, BytesMut};

        let initial_vec: Vec<u8> = vec![];
        let result = (0..buffer_size)
            .map(|_| rand::random::<u8>())
            .fold(initial_vec, |mut a, b| {
                a.put_u8(b);
                a
            });

        BytesMut::from(result.as_slice())
    }

    pub mod macros {
        #[macro_export]
        macro_rules! is_type {
            ($value:expr, $pattern:pat) => {
                match &$value {
                    $pattern => {
                        true
                    },
                    _ => {
                        false
                    }
                }
            }
        }
    }
}