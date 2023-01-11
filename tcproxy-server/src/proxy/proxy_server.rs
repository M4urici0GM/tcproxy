use std::sync::Arc;
use tcproxy_core::{TcpFrame};
use tokio::sync::{mpsc, OwnedSemaphorePermit};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use tcproxy_core::framing::{SocketConnected};
use tcproxy_core::tcp::{SocketListener, TcpStream};
use tcproxy_core::Result;

use crate::tcp::{RemoteConnection};
use crate::ClientState;

pub struct ProxyServer {
    target_port: u16,
    listener: Box<dyn SocketListener + 'static>,
    proxy_state: Arc<ClientState>,
    client_sender: Sender<TcpFrame>,
}

impl ProxyServer {
    pub fn new<T>(
        target_port: &u16,
        state: &Arc<ClientState>,
        sender: &Sender<TcpFrame>,
        listener: T,
    ) -> Self
        where T: SocketListener + 'static {
        Self {
            target_port: *target_port,
            proxy_state: state.clone(),
            client_sender: sender.clone(),
            listener: Box::new(listener),
        }
    }

    pub fn spawn(mut self, cancellation_token: CancellationToken) {
        tokio::spawn(async move {
            let port_manager = self.proxy_state.get_port_manager().clone();
            let token = cancellation_token.child_token();

            tokio::select! {
                _ = self.start() => {},
                _ = token.cancelled() => {},
            }

            debug!("proxy server finished.");
            port_manager.remove_port(self.target_port);
        });
    }

    async fn start(&mut self) -> Result<()> {
        let semaphore = Arc::new(Semaphore::new(120));

        loop {
            let permit = semaphore.clone()
                .acquire_owned()
                .await
                .unwrap();

            let connection = self.listener.accept().await?;
            self.spawn_remote_connection(connection, permit).await?;
        }
    }

    async fn spawn_remote_connection(
        &self,
        connection: TcpStream,
        permit: OwnedSemaphorePermit,
    ) -> Result<()> {
        let (connection_id, receiver) = self.create_connection_state();
        let remote_connection = RemoteConnection::new(
            &connection_id,
            permit,
            &self.client_sender);

        tokio::spawn(async move {
            let _ = remote_connection.start(connection, receiver).await;
        });

        self.send_incoming_connection_frame(&connection_id).await?;
        Ok(())
    }

    async fn send_incoming_connection_frame(&self, connection_id: &u32) -> Result<()>{
        self.client_sender
            .send(TcpFrame::SocketConnected(SocketConnected::new(connection_id)))
            .await?;

        Ok(())
    }

    fn create_connection_state(&self) -> (u32, Receiver<Vec<u8>>) {
        let connection_manager = self.proxy_state.get_connection_manager();

        let (connection_sender, connection_receiver) = mpsc::channel::<Vec<u8>>(100);
        let connection_id = connection_manager.insert_connection(
            connection_sender,
            CancellationToken::new());

        (connection_id, connection_receiver)
    }
}



