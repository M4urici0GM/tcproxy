use std::net::SocketAddr;

use tokio::io::{AsyncRead, AsyncWrite};

pub trait SocketConnection: Sync + Send {
    fn split(
        self,
    ) -> (
        Box<dyn AsyncRead + Send + Unpin>,
        Box<dyn AsyncWrite + Send + Unpin>,
    );

    fn addr(&self) -> SocketAddr;
}
