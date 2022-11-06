mod socket_connection;
mod socket_listener;
mod tcp_listener;
mod tcp_stream;
mod stream_reader;

pub use socket_connection::*;
pub use socket_listener::*;
pub use tcp_listener::*;
pub use tcp_stream::*;
pub use stream_reader::*;

pub type ISocketListener = Box<dyn SocketListener>;
