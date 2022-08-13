use async_trait::async_trait;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::net::{TcpListener as TokioTcpListener, TcpStream};
use tracing::{debug, error};

use crate::Result;

#[derive(Debug)]
pub struct TcpListener {
    inner: TokioTcpListener,
}

#[async_trait]
pub trait SocketListener: Debug + Sync + Send {

    /// Creates a new SocketListener, which will be bound to the specific address.
    async fn bind(addr: SocketAddr) -> Result<Self>
    where
        Self: Sized;

    /// Accepts new incoming connection from this listener.
    async fn accept(&self) -> Result<(TcpStream, SocketAddr)>;

    fn listen_ip(&self) -> Result<SocketAddr>;
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

    async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        let mut backoff = 1;
        loop {
            match self.inner.accept().await {
                Ok((inner, addr)) => {
                    debug!("New socket {} connected.", addr);
                    return Ok((inner, addr));
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
