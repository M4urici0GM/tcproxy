mod command;
mod frame_error;
mod tcp_frame;

pub mod tcp;
pub mod transport;
pub mod framing;
pub mod io;
pub mod config;

pub use command::*;
pub use frame_error::*;
pub use tcp_frame::*;

pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
pub type Result<T> = std::result::Result<T, Error>;


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