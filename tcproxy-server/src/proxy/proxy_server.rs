use bytes::BytesMut;
use std::sync::Arc;
use tcproxy_core::TcpFrame;
use tokio::sync::{mpsc, OwnedSemaphorePermit};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};
use uuid::Uuid;

use tcproxy_core::tcp::{SocketListener, TcpStream};
use tcproxy_core::Result;

use crate::tcp::RemoteConnection;
use crate::ClientState;

pub struct ProxyServer {
    pub(crate) listener: Box<dyn SocketListener>,
    pub(crate) target_port: u16,
    pub(crate) proxy_state: Arc<ClientState>,
    pub(crate) client_sender: Sender<TcpFrame>,
    pub(crate) cancellation_token: CancellationToken,
}

impl ProxyServer {
    pub fn spawn(mut self) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let port_manager = self.proxy_state.get_port_manager();
            let token = self.cancellation_token.child_token();

            tokio::select! {
                _ = self.start() => {},
                _ = token.cancelled() => {},
            }

            debug!("proxy server finished.");
            port_manager.remove_port(self.target_port);
            Ok(())
        })
    }

    async fn accept_connection(&mut self) -> Result<TcpStream> {
        match self.listener.accept().await {
            Ok(stream) => {
                debug!(
                    "received new connection on proxy {} from {}",
                    self.listener.listen_ip()?,
                    stream.addr
                );
                Ok(stream)
            },
            Err(err) => {
                let ip = self.listener.listen_ip()?;
                error!("failed to accept socket. {}: {}", ip, err);
                debug!("closing proxy listener {}: {}", ip, err);
                Err(err)
            }
        }
    }

    async fn start(&mut self) -> Result<()> {
        let semaphore = Arc::new(Semaphore::new(120));
        let connection_manager = self.proxy_state.get_connection_manager();

        loop {
            let permit = semaphore.clone()
                .acquire_owned()
                .await
                .unwrap();

            let connection = self.accept_connection().await?;

            let connection_id = Uuid::new_v4();
            let (connection_sender, connection_receiver) = mpsc::channel::<BytesMut>(100);

            connection_manager.insert_connection(
                connection_id,
                connection_sender,
                CancellationToken::new(),
            );

            self.spawn_remote_connection(permit, connection, connection_id, connection_receiver).await?;
        }
    }

    async fn spawn_remote_connection(
        &mut self,
        permit: OwnedSemaphorePermit,
        connection: TcpStream,
        connection_id: Uuid,
        connection_receiver: Receiver<BytesMut>,
    ) -> Result<()> {
        self.client_sender
            .send(TcpFrame::IncomingSocket { connection_id })
            .await?;

        let mut remote_connection = RemoteConnection::new(
            permit,
            connection.addr,
            connection_id,
            &self.client_sender);

        tokio::spawn(async move {
            let _ = remote_connection
                .start(connection, connection_receiver)
                .await;

            debug!("Remote connection task was cancelled.")
        });

        Ok(())
    }
}
