mod duplex_stream;
mod server;
mod app;
mod args;
pub mod codec;

pub use duplex_stream::DuplexTcpStream;
pub use args::AppArguments;
pub use server::Server;

pub type Error = Box<dyn std::error::Error + Sync + Send>;
pub type Result<T> = std::result::Result<T, Error>;