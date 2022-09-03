use std::fmt::Debug;
use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{info, debug};
use tcproxy_core::Result;

use tcproxy_core::tcp::SocketListener;

use crate::{AppArguments, ProxyState};
use crate::proxy::Connection;

#[derive(Debug)]
pub struct Server {
    args: Arc<AppArguments>,
    server_listener: Box<dyn SocketListener>,
}

impl Server {
    pub fn new(args: AppArguments, listener: Box<dyn SocketListener>) -> Self {
        Self {
            args: Arc::new(args),
            server_listener: listener,
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
        let port_range = self.args.parse_port_range()?;
        let listen_ip = self.args.parse_ip()?;

        loop {
            let (socket, addr) = self.server_listener.accept().await?;

            let proxy_state = Arc::new(ProxyState::new(&port_range));
            let cancellation_token = cancellation_token.child_token();
            let mut proxy_client = Connection::new(listen_ip, proxy_state.clone());

            tokio::spawn(async move {
                match proxy_client.start_streaming(socket, cancellation_token).await {
                    Ok(_) => debug!("Socket {} has been closed gracefully.", addr),
                    Err(err) => debug!("Socket {} has been disconnected with error.. {}", addr, err)
                };
            });
        }
    }
}