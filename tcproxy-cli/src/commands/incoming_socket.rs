use async_trait::async_trait;
use bytes::BytesMut;
use std::sync::Arc;
use tcproxy_core::{AsyncCommand, Result, TcpFrame};
use tokio::sync::mpsc::{self, Sender};
use tokio_util::sync::CancellationToken;
use tracing::debug;
use tcproxy_core::framing::LocalConnectionDisconnected;

use crate::{client_state::ClientState, ListenArgs, LocalConnection};

/// issued when a remote socket connects to server.
pub struct IncomingSocketCommand {
    connection_id: u32,
    client_sender: Sender<TcpFrame>,
    state: Arc<ClientState>,
    args: Arc<ListenArgs>,
}

impl IncomingSocketCommand {
    pub fn new(
        id: &u32,
        sender: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
        args: &Arc<ListenArgs>,
    ) -> Self {
        Self {
            connection_id: *id,
            args: args.clone(),
            state: state.clone(),
            client_sender: sender.clone(),
        }
    }
}

#[async_trait]
impl AsyncCommand for IncomingSocketCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        debug!("new connection received!");
        let (connection_sender, reader) = mpsc::channel::<BytesMut>(1000);
        let token = CancellationToken::new();
        let cancellation_token = token.child_token();

        self.state
            .insert_connection(&self.connection_id, connection_sender, token);

        let target_ip = self.args.parse_socket_addr();
        let connection_id = self.connection_id;
        let sender = self.client_sender.clone();
        let mut local_connection =
            LocalConnection::new(self.connection_id, &self.client_sender, target_ip);

        tokio::spawn(async move {
            let _ = local_connection
                .read_from_local_connection(reader, cancellation_token.child_token())
                .await;


            debug!("Local connection socket finished.");
            let _ = sender
                .send(TcpFrame::LocalConnectionDisconnected(LocalConnectionDisconnected::new(&connection_id)))
                .await;
        });

        Ok(())
    }
}
