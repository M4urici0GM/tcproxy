use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::net::TcpListener as TokioTcpListener;
use tokio_native_tls::native_tls::{Identity, TlsAcceptor};
use tokio_native_tls::TlsAcceptor as TokioTlsAcceptor;
use tracing::error;

use crate::stream::Stream;
use crate::tcp::SocketListener;
use crate::Result;


#[derive(Debug)]
pub struct TcpListener {
    inner: TokioTcpListener,
    acceptor: Option<Arc<TokioTlsAcceptor>>,
}

pub struct RemoteConnection{
    pub stream: Stream,
    remote_addr: SocketAddr
}

impl RemoteConnection {
    pub fn new(stream: Stream, remote_addr: SocketAddr) -> Self {
        Self {
            stream,
            remote_addr,
        }
    }
    
    pub fn remote_addr(&self) -> &SocketAddr {
        &self.remote_addr
    }
}

#[async_trait]
impl SocketListener for TcpListener {
    async fn bind(addr: SocketAddr, identity: Option<Identity>) -> Result<Self>
    where
        Self: Sized,
    {
        Ok(TcpListener {
            inner: TokioTcpListener::bind(addr).await?,
            acceptor: match identity {
                None => None,
                Some(identity) => {
                    let internal_acceptor = TlsAcceptor::new(identity)?;
                    Some(Arc::new(TokioTlsAcceptor::from(internal_acceptor)))
                }
            },
        })
    }

    async fn accept(&self) -> Result<RemoteConnection> {
        let mut backoff = 1;
        loop {
            match self.inner.accept().await {
                Ok((stream, addr)) => {
                    let result = match &self.acceptor {
                        None => Stream::new(stream),
                        Some(acceptor) => Stream::new(acceptor.accept(stream).await?),
                    };

                    return Ok(RemoteConnection { stream: result, remote_addr: addr });
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
            }
        }
    }

    fn listen_ip(&self) -> Result<SocketAddr> {
        Ok(self.inner.local_addr()?)
    }
}
