use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use bytes::BytesMut;
use tcproxy_core::TcpFrame;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use tracing::{debug, error};

use tcproxy_core::Result;

use crate::ProxyState;
use crate::tcp::{ListenerUtils, RemoteConnection};

pub struct ProxyServer {
    pub(crate) listener: ListenerUtils,
    pub(crate) target_ip: IpAddr,
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

    async fn accept_connection(
        &mut self,
        listener: &TcpListener,
    ) -> Result<(TcpStream, SocketAddr)> {
        match self.listener.accept(listener).await {
            Ok(connection) => Ok(connection),
            Err(err) => {
                error!(
                    "failed to accept socket. {}: {}",
                    self.listener.listen_ip(),
                    err
                );
                debug!(
                    "closing proxy listener {}: {}",
                    self.listener.listen_ip(),
                    err
                );
                Err(err.into())
            }
        }
    }

    async fn start(&mut self) -> Result<()> {
        let (listener, _) = self.bind().await?;
        let _ = self
            .client_sender
            .send(TcpFrame::ClientConnectedAck {
                port: self.target_port,
            })
            .await;

        let semaphore = Arc::new(Semaphore::new(120));
        loop {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let (connection, socket_addr) = self.accept_connection(&listener).await?;

            debug!(
                "received new connection on proxy {} from {}",
                self.listener.listen_ip(),
                socket_addr
            );

            let connection_id = Uuid::new_v4();
            let (connection_sender, connection_receiver) = mpsc::channel::<BytesMut>(100);

            self.proxy_state.connections.insert_connection(connection_id, connection_sender, CancellationToken::new());
            self
                .client_sender
                .send(TcpFrame::IncomingSocket { connection_id })
                .await?;

            let _ = RemoteConnection::new(permit, socket_addr, connection_id, &self.client_sender)
                    .spawn(connection, connection_receiver);
        }
    }

    async fn bind(&self) -> Result<(TcpListener, SocketAddr)> {
        match self.listener.bind().await {
            Ok(listener) => {
                let addr = listener.local_addr().unwrap();
                Ok((listener, addr))
            }
            Err(err) => {
                error!(
                    "failed to listen to {}:{} {}",
                    self.target_ip, self.target_port, err
                );
                Err(err.into())
            }
        }
    }
}