mod duplex_stream;
mod server;
mod args;
mod port_manager;
pub mod codec;
pub mod proxy;
pub mod tcp;

pub use duplex_stream::DuplexTcpStream;
pub use args::AppArguments;
pub use server::Server;
pub use port_manager::PortManager;

pub type Error = Box<dyn std::error::Error + Sync + Send  + 'static>;
pub type Result<T> = std::result::Result<T, Error>;