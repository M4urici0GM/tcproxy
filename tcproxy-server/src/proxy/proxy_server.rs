use bytes::BytesMut;
use std::net::SocketAddr;
use std::sync::Arc;
use tcproxy_core::TcpFrame;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};
use uuid::Uuid;

use tcproxy_core::tcp::SocketListener;
use tcproxy_core::Result;

use crate::tcp::{RemoteConnection, Socket};
use crate::ProxyState;

pub struct ProxyServer {
    pub(crate) listener: Box<dyn SocketListener>,
    pub(crate) target_port: u16,
    pub(crate) proxy_state: Arc<ProxyState>,
    pub(crate) client_sender: Sender<TcpFrame>,
    pub(crate) cancellation_token: CancellationToken,
}

impl ProxyServer {
    pub fn spawn(mut self) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            // TODO: send nack message to client if bind fails.

            let token = self.cancellation_token.child_token();
            tokio::select! {
                _ = self.start() => {},
                _ = token.cancelled() => {},
            }

            debug!("proxy server finished.");
            self.proxy_state.ports.remove_port(self.target_port);
            Ok(())
        })
    }

    async fn accept_connection(&mut self) -> Result<(TcpStream, SocketAddr)> {
        match self.listener.accept().await {
            Ok(connection) => Ok(connection),
            Err(err) => {
                let ip = self.listener.listen_ip()?;
                error!("failed to accept socket. {}: {}", ip, err);
                debug!("closing proxy listener {}: {}", ip, err);
                Err(err.into())
            }
        }
    }

    async fn start(&mut self) -> Result<()> {
        let _ = self
            .client_sender
            .send(TcpFrame::ClientConnectedAck {
                port: self.target_port,
            })
            .await;

        let semaphore = Arc::new(Semaphore::new(120));
        loop {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let (connection, socket_addr) = self.accept_connection().await?;

            debug!(
                "received new connection on proxy {} from {}",
                self.listener.listen_ip()?,
                socket_addr
            );

            let connection_id = Uuid::new_v4();
            let (connection_sender, connection_receiver) = mpsc::channel::<BytesMut>(100);

            self.proxy_state.connections.insert_connection(
                connection_id,
                connection_sender,
                CancellationToken::new(),
            );
            self.client_sender
                .send(TcpFrame::IncomingSocket { connection_id })
                .await?;

            let mut remote_connection = RemoteConnection::new(permit, socket_addr, connection_id, &self.client_sender);

            let socket = Socket { inner: connection };
            tokio::spawn(async move {
                let _ = remote_connection.start(socket, connection_receiver).await;
            });
        }
    }
}
