use std::net::SocketAddr;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream as TokioTcpStream,
};

use crate::tcp::SocketConnection;

#[derive(Debug)]
pub struct TcpStream {
    pub addr: SocketAddr,
    pub inner: TokioTcpStream,
}

impl TcpStream {
    pub fn new(inner: TokioTcpStream, addr: SocketAddr) -> Self {
        Self { inner, addr }
    }

    pub async fn connect(addr: SocketAddr) -> std::io::Result<Self> {
        let stream = TokioTcpStream::connect(addr).await?;
        Ok(Self::new(stream, addr))
    }
}

impl SocketConnection for TcpStream {
    fn split(
        self,
    ) -> (
        Box<dyn AsyncRead + Send + Unpin>,
        Box<dyn AsyncWrite + Send + Unpin>,
    ) {
        let (reader, writer) = self.inner.into_split();

        (Box::new(reader), Box::new(writer))
    }

    fn addr(&self) -> SocketAddr {
        self.addr
    }
}
