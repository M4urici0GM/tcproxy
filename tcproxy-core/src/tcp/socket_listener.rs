use std::{fmt::Debug, net::SocketAddr};

use crate::Result;
use async_trait::async_trait;
use mockall::automock;
use tokio_native_tls::native_tls::Identity;

use super::RemoteConnection;

#[automock]
#[async_trait]
pub trait SocketListener: Debug + Sync + Send {
    /// Creates a new SocketListener, which will be bound to the specific address.
    async fn bind(addr: SocketAddr, identity: Option<Identity>) -> Result<Self>
    where
        Self: Sized;

    /// Accepts new incoming connection from this listener.
    async fn accept(&self) -> Result<RemoteConnection>;

    fn listen_ip(&self) -> Result<SocketAddr>;
}
