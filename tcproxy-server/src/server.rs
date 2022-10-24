use std::fmt::Debug;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tcproxy_core::Result;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use tcproxy_core::tcp::{SocketConnection, SocketListener};
use crate::managers::FeatureManager;

use crate::proxy::ClientConnection;

/// Represents the server application
pub struct Server {
    feature_manager: Box<dyn FeatureManager>,
    server_listener: Box<dyn SocketListener>,
}

impl Server {
    pub fn new<TListener, TFeatureManager>(feature_manager: TFeatureManager, listener: TListener) -> Self
    where
        TListener: SocketListener + 'static,
        TFeatureManager: FeatureManager + 'static
    {
        Self {
            feature_manager: Box::new(feature_manager),
            server_listener: Box::new(listener),
        }
    }

    pub async fn run(&mut self, shutdown_signal: impl Future) -> Result<()> {
        let cancellation_token = CancellationToken::new();
        tokio::select! {
            _ = self.start(cancellation_token.child_token()) => {},
            _ = shutdown_signal => {
                info!("server is being shut down.");
                cancellation_token.cancel();
            }
        };

        Ok(())
    }

    pub fn get_listen_ip(&self) -> Result<SocketAddr> {
        self.server_listener.listen_ip()
    }

    async fn start(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        loop {
            let socket = self.server_listener.accept().await?;
            let cancellation_token = cancellation_token.child_token();

            self.spawn_proxy_connection(socket, cancellation_token);
        }
    }

    fn spawn_proxy_connection<T>(
        &self,
        socket: T,
        cancellation_token: CancellationToken,
    ) -> JoinHandle<Result<()>>
    where
        T: SocketConnection + 'static,
    {
        let mut proxy_client = ClientConnection::new(&self.server_config);
        tokio::spawn(async move {
            let socket_addr = socket.addr();

            match proxy_client
                .start_streaming(socket, cancellation_token)
                .await
            {
                Ok(_) => debug!("Socket {} has been closed gracefully.", socket_addr),
                Err(err) => debug!(
                    "Socket {} has been disconnected with error.. {}",
                    socket_addr, err
                ),
            };

            Ok(())
        })
    }
}
