mod frame_reader;
mod tcp_stream;
mod frame_error;
mod command;

pub use frame_reader::*;
pub use tcp_stream::*;
pub use frame_error::*;
pub use command::*;

pub type Error = Box<dyn std::error::Error + Sync + Send  + 'static>;
pub type Result<T> = std::result::Result<T, Error>;
