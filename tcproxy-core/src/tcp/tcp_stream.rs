use std::net::SocketAddr;
use tokio::{net::TcpStream as TokioTcpStream, io::{AsyncRead, AsyncWrite}};

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

