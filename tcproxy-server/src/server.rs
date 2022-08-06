use std::future::Future;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{info, debug};
use tcproxy_core::Result;

use crate::{AppArguments, ProxyState};
use crate::proxy::Connection;
use crate::tcp::ListenerUtils;

#[derive(Debug)]
pub struct Server {
    args: Arc<AppArguments>,
    server_listener: ListenerUtils,
}

impl Server {
    pub fn new(args: AppArguments) -> Self {
        let ip = args.parse_ip().unwrap();
        let port = args.port();

        Self {
            args: Arc::new(args),
            server_listener: ListenerUtils::new(ip, port),
        }
    }

    pub async fn run(&self, shutdown_signal: impl Future) -> Result<()> {
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

    async fn start(&self, cancellation_token: CancellationToken) -> Result<()> {
        let tcp_listener = self.server_listener.bind().await?;
        let port_range = self.args.parse_port_range()?;
        let listen_ip = self.args.parse_ip()?;

        while !cancellation_token.is_cancelled() {
            let (socket, addr) = self.server_listener.accept(&tcp_listener).await?;

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

        Ok(())
    }
}