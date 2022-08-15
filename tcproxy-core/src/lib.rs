mod tcp_frame;
mod frame_error;
mod command;

pub mod tcp;
pub mod transport;

pub use tcp_frame::*;
pub use frame_error::*;
pub use command::*;

pub type Error = Box<dyn std::error::Error + Sync + Send  + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

