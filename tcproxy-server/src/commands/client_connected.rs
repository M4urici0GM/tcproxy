use async_trait::async_trait;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use clap::command;
use tcproxy_core::tcp::{SocketListener, TcpListener};
use tcproxy_core::{AsyncCommand, Command, Result, TcpFrame};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

use crate::proxy::ProxyServer;
use crate::ClientState;
use crate::managers::PortError;

pub struct ClientConnectedCommand {
    target_ip: IpAddr,
    client_sender: Sender<TcpFrame>,
    state: Arc<ClientState>,
    cancellation_token: CancellationToken,
}

impl ClientConnectedCommand {
    pub fn new(
        target_ip: &IpAddr,
        sender: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
        cancellation_token: &CancellationToken,
    ) -> Self {
        Self {
            target_ip: *target_ip,
            client_sender: sender.clone(),
            state: state.clone(),
            cancellation_token: cancellation_token.child_token(),
        }
    }

    pub fn boxed_new(
        target_ip: &IpAddr,
        sender: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
        cancellation_token: &CancellationToken
    ) -> Box<Self> {
        let local_self = ClientConnectedCommand::new(target_ip, sender, state, cancellation_token);
        Box::new(local_self)
    }

    async fn get_available_port(&self) -> Result<u16> {
        match self.state.ports.get_port().await {
            Ok(port) => Ok(port),
            Err(PortError::PortLimitReached(err)) => {
                debug!("server cannot listen to more ports. port limit reached.");
                self.client_sender.send(TcpFrame::PortLimitReached).await?;
                return Err(err.into());
            }
            Err(err) => {
                error!("failed when trying to reserver a port for proxy server: {}", err);
                return Err(err.into());
            },
        }
    }
}

#[async_trait]
impl AsyncCommand for ClientConnectedCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        let target_port = self.get_available_port().await?;
        let target_ip = SocketAddr::new(self.target_ip, target_port);
        let listener = match TcpListener::bind(target_ip).await {
            Ok(listener) => listener,
            Err(err) => {
                error!("error when trying to spawn tcp proxy listener. {}", err);
                let _ = self
                    .client_sender
                    .send(TcpFrame::FailedToCreateProxy)
                    .await;

                return Err(err);
            }
        };

        let proxy_server = ProxyServer {
            target_port,
            listener: Box::new(listener),
            client_sender: self.client_sender.clone(),
            proxy_state: self.state.clone(),
            cancellation_token: self.cancellation_token.child_token(),
        };

        let _ = proxy_server.spawn();
        let _ = self
            .client_sender
            .send(TcpFrame::ClientConnectedAck {
                port: target_port,
            })
            .await;

        Ok(())
    }
}
