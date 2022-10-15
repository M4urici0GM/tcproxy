use async_trait::async_trait;
use mockall::automock;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::{TcpListener as TokioTcpListener, TcpStream as TokioTcpStream},
};
use tracing::{debug, error};

use crate::Result;

#[derive(Debug)]
pub struct TcpListener {
    inner: TokioTcpListener,
}

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

#[automock]
#[async_trait]
pub trait SocketListener: Debug + Sync + Send {
    /// Creates a new SocketListener, which will be bound to the specific address.
    async fn bind(addr: SocketAddr) -> Result<Self>
    where
        Self: Sized;

    /// Accepts new incoming connection from this listener.
    async fn accept(&self) -> Result<TcpStream>;

    fn listen_ip(&self) -> Result<SocketAddr>;
}

pub trait SocketConnection: Sync + Send {
    fn split(
        self,
    ) -> (
        Box<dyn AsyncRead + Send + Unpin>,
        Box<dyn AsyncWrite + Send + Unpin>,
    );

    fn addr(&self) -> SocketAddr;
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
        self.addr.clone()
    }
}

#[async_trait]
impl SocketListener for TcpListener {
    async fn bind(addr: SocketAddr) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(TcpListener {
            inner: TokioTcpListener::bind(addr).await?,
        })
    }

    async fn accept(&self) -> Result<TcpStream> {
        let mut backoff = 1;
        loop {
            match self.inner.accept().await {
                Ok((inner, addr)) => {
                    debug!("New socket {} connected.", addr);
                    return Ok(TcpStream::new(inner, addr));
                }
                Err(err) => {
                    error!("Failed to accept new socket. retrying.. {}", err);
                    if backoff > 64 {
                        error!("Failed to accept new socket. aborting.. {}", err);
                        return Err(err.into());
                    }

                    backoff *= 2;
                }
            };
        }
    }

    fn listen_ip(&self) -> Result<SocketAddr> {
        Ok(self.inner.local_addr()?)
    }
}
