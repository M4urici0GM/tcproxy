use std::future::Future;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{info, debug};

use crate::{AppArguments, Result};
use crate::proxy::ProxyClient;
use crate::tcp::Listener;

#[derive(Debug)]
pub struct Server {
    args: Arc<AppArguments>,
    server_listener: Listener,
}

impl Server {
    pub fn new(args: AppArguments) -> Self {
        let ip = args.parse_ip().unwrap();
        let port = args.port();

        Self {
            args: Arc::new(args),
            server_listener: Listener { ip, port },
        }
    }

    async fn start(&self, cancellation_token: CancellationToken) -> Result<()> {
        let tcp_listener = self.server_listener.bind().await?;
        let available_proxies: Arc<Mutex<Vec<u16>>> = Arc::new(Mutex::new(Vec::new()));
        let port_range = self.args.parse_port_range()?;
        let listen_ip = self.args.parse_ip()?;

        while !cancellation_token.is_cancelled() {
            let (socket, addr) = self.server_listener.accept(&tcp_listener).await?;
            let mut connection_handler = ProxyClient::new(socket, port_range, listen_ip, available_proxies.clone());

            let cancellation_token = cancellation_token.child_token();
            tokio::spawn(async move {
                match connection_handler.start_streaming(cancellation_token).await {
                    Ok(_) => {
                        debug!("Socket {} has been closed gracefully.", addr);
                    }
                    Err(err) => {
                        debug!("Socket {} has been disconnected with error.. {}", addr, err);
                    }
                };
            });
        }

        Ok(())
    }


    pub async fn run(&self, shutdown_signal: impl Future) -> Result<()> {
        let cancellation_token = CancellationToken::new();
        tokio::select! {
            _ = self.start(cancellation_token.child_token()) => {

            },
            _ = shutdown_signal => {
                info!("server is being shut down.");
            }
        };

        cancellation_token.cancel();
        Ok(())
    }
}