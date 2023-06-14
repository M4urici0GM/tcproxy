use std::net::SocketAddr;

use async_trait::async_trait;
use tokio::net::TcpListener as TokioTcpListener;
use tracing::{debug, error};

use crate::tcp::SocketListener;
use crate::Result;

use super::TcpStream;

#[derive(Debug)]
pub struct TcpListener {
    inner: TokioTcpListener,
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
                    error!(
                        "Failed to accept new socket at listener {}. retrying.. {}",
                        self.inner.local_addr()?,
                        err
                    );
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
