use std::{sync::Arc, net::{SocketAddrV4, Ipv4Addr}, str::FromStr};

use async_trait::async_trait;
use bytes::BytesMut;
use tcproxy_core::{Result, TcpFrame};
use tokio::sync::mpsc::{self, Sender};
use tokio_util::sync::CancellationToken;
use tracing::debug;
use uuid::Uuid;

use tcproxy_core::Command;

use crate::{LocalConnection, client_state::ClientState, ClientArgs};

/// issued when a remote socket connects to server.
pub struct IncomingSocketCommand {
    connection_id: Uuid,
    client_sender: Sender<TcpFrame>,
    state: Arc<ClientState>,
    args: Arc<ClientArgs>,
}

impl IncomingSocketCommand {
    pub fn new(id: Uuid, sender: &Sender<TcpFrame>, state: &Arc<ClientState>, args: &Arc<ClientArgs>) -> Self {
        Self {
            connection_id: id,
            args: args.clone(),
            state: state.clone(),
            client_sender: sender.clone(),
        }
    }
}

#[async_trait]
impl Command for IncomingSocketCommand {
    async fn handle(&mut self) -> Result<()> {
        debug!("new connection received!");
        let (connection_sender, reader) = mpsc::channel::<BytesMut>(1000);
        let token = CancellationToken::new();
        let cancellation_token = token.child_token();

        self.state.insert_connection(self.connection_id, connection_sender, token);

        let target_ip = self.args.parse_socket_addr();
        let connection_id = self.connection_id.clone();
        let sender = self.client_sender.clone();
        let mut local_connection = LocalConnection::new(self.connection_id, &self.client_sender, target_ip);

        tokio::spawn(async move {
            let _ = local_connection
                .read_from_local_connection(reader, cancellation_token.child_token())
                .await;

            debug!("Local connection socket finis`hed.");
            let _ = sender
                .send(TcpFrame::LocalClientDisconnected { connection_id })
                .await;
        });

        Ok(())
    }
}
