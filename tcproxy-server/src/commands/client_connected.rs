use async_trait::async_trait;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tcproxy_core::tcp::{SocketListener, TcpListener};
use tcproxy_core::{AsyncCommand, Result, TcpFrame};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::proxy::ProxyServer;
use crate::ClientState;

pub struct ClientConnectedCommand {
    pub(crate) target_ip: IpAddr,
    pub(crate) sender: Sender<TcpFrame>,
    pub(crate) state: Arc<ClientState>,
    pub(crate) cancellation_token: CancellationToken,
}

#[async_trait]
impl AsyncCommand for ClientConnectedCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        let target_port = match self.state.ports.get_port().await {
            Ok(port) => port,
            Err(err) => {
                debug!("server cannot listen to more ports. port limit reached.");
                self.sender.send(TcpFrame::PortLimitReached).await?;
                return Err(err);
            }
        };

        let target_ip = SocketAddr::new(self.target_ip, target_port);
        let listener = TcpListener::bind(target_ip).await?;
        let proxy_server = ProxyServer {
            target_port,
            listener: Box::new(listener),
            client_sender: self.sender.clone(),
            proxy_state: self.state.clone(),
            cancellation_token: self.cancellation_token.child_token(),
        };

        let _ = proxy_server.spawn();

        Ok(())
    }
}
