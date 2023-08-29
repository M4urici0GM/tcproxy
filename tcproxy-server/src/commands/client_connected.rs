use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use tcproxy_core::tcp::SocketListener;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use tcproxy_core::framing::ClientConnectedAck;
use tcproxy_core::{AsyncCommand, Result, TcpFrame};

use crate::proxy::ProxyServer;
use crate::ClientState;

pub struct ClientConnectedCommand {
    client_sender: Sender<TcpFrame>,
    client_state: Arc<ClientState>,
}

impl ClientConnectedCommand {
    pub fn new(sender: &Sender<TcpFrame>, state: &Arc<ClientState>) -> Self {
        Self {
            client_sender: sender.clone(),
            client_state: Arc::clone(state),
        }
    }

    pub fn boxed_new(sender: &Sender<TcpFrame>, state: &Arc<ClientState>) -> Box<Self> {
        let local_self = ClientConnectedCommand::new(sender, state);
        Box::new(local_self)
    }
}

#[async_trait]
impl AsyncCommand for ClientConnectedCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        debug!("received connection client command");

        let port_permit = self
            .client_state
            .get_port_manager()
            .reserve_port(&1234, "")?;

        let target_addr = self.client_state
            .get_server_config()
            .get_listen_ip();

        debug!("spawning new TcpListener at {}", &target_addr);

        let target_socket = SocketAddr::new(target_addr, *port_permit.port());
        let listener = tcproxy_core::tcp::TcpListener::bind(target_socket, None).await?;
        let proxy_server = ProxyServer::new(port_permit, &self.client_state, &self.client_sender, listener);

        tokio::spawn(async move {
            let _ = proxy_server.spawn(CancellationToken::new());
            // TODO: send message to client when server shuts down for any reason.
        });

        info!("new TcpListener running at {}", &target_socket);

        self.client_sender
            .send(TcpFrame::ClientConnectedAck(ClientConnectedAck::new(&target_socket.port())))
            .await?;

        Ok(())
    }
}
