use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use tcproxy_core::tcp::{SocketListener, TcpListener};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use tcproxy_core::framing::{ClientConnected, ClientConnectedAck};
use tcproxy_core::{Result, TcpFrame};

use super::NewFrameHandler;
use crate::proxy::ProxyServer;
use crate::ClientState;

pub struct ClientConnectedHandler(ClientConnected);

impl From<ClientConnected> for ClientConnectedHandler {
    fn from(value: ClientConnected) -> Self { Self(value) }
}

impl Into<Box<dyn NewFrameHandler>> for ClientConnectedHandler {
    fn into(self) -> Box<dyn NewFrameHandler> {
        Box::new(self)
    }
}

#[async_trait]
impl NewFrameHandler for ClientConnectedHandler {
    async fn execute(
        &self,
        tx: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
    ) -> Result<Option<TcpFrame>> {
        tracing::debug!("received connection client command");

        let port_permit = state.get_port_manager().reserve_port(&1234, "")?;
        let target_addr = state.get_server_config().get_listen_ip();

        tracing::debug!("spawning new TcpListener at {}", &target_addr);

        let target_socket = SocketAddr::new(target_addr, *port_permit.port());
        let listener = TcpListener::bind(target_socket, None).await?;
        let proxy_server = ProxyServer::new(port_permit, &state, &tx, listener);

        tokio::spawn(async move {
            let _ = proxy_server.spawn(CancellationToken::new());
            // TODO: send message to client when server shuts down for any reason.
        });

        tracing::info!("new TcpListener running at {}", &target_socket);

        Ok(Some(TcpFrame::from(ClientConnectedAck::new(
            &target_socket.port(),
        ))))
    }
}
