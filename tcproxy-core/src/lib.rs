mod command;
mod frame_error;
mod tcp_frame;

pub mod tcp;
pub mod transport;
pub mod framing;
pub mod io;

pub use command::*;
pub use frame_error::*;
pub use tcp_frame::*;

pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
pub type Result<T> = std::result::Result<T, Error>;


pub mod test_util {
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