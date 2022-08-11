use async_trait::async_trait;
use std::{net::Ipv4Addr, sync::Arc};
use std::net::IpAddr;
use tcproxy_core::{Command, Result, TcpFrame};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::proxy::ProxyServer;
use crate::tcp::ListenerUtils;
use crate::ProxyState;

pub struct ClientConnectedCommand {
    pub(crate) target_ip: IpAddr,
    pub(crate) sender: Sender<TcpFrame>,
    pub(crate) state: Arc<ProxyState>,
    pub(crate) cancellation_token: CancellationToken,
}

#[async_trait]
impl Command for ClientConnectedCommand {
    type Output = ();

    async fn handle(&mut self) -> Result<()> {
        let target_port = match self.state.ports.get_port().await {
            Ok(port) => port,
            Err(err) => {
                debug!("server cannot listen to more ports. port limit reached.");
                self.sender.send(TcpFrame::PortLimitReached).await?;
                return Err(err);
            }
        };

        let listener = ListenerUtils::new(self.target_ip, target_port);
        let proxy_server = ProxyServer {
            listener,
            target_port,
            client_sender: self.sender.clone(),
            proxy_state: self.state.clone(),
            target_ip: self.target_ip.clone(),
            cancellation_token: self.cancellation_token.child_token()
        };

        let _ = proxy_server.spawn();

        Ok(())
    }
}
