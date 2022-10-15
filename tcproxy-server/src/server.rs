use std::fmt::Debug;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::ops::Range;
use tcproxy_core::Result;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use tcproxy_core::tcp::{SocketConnection, SocketListener, TcpStream};

use crate::proxy::ClientConnection;

#[derive(Debug)]
pub struct Server {
    port_range: Range<u16>,
    listen_ip: IpAddr,
    server_listener: Box<dyn SocketListener>,
}

impl Server {
    pub fn new<T>(port_range: &Range<u16>, listen_ip: &IpAddr, listener: T) -> Self
    where
        T: SocketListener + 'static,
    {
        Self {
            server_listener: Box::new(listener),
            listen_ip: listen_ip.clone(),
            port_range: port_range.clone(),
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
        let mut proxy_client = ClientConnection::new(&self.listen_ip, &self.port_range);
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
