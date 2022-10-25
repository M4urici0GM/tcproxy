use async_trait::async_trait;
use std::net::IpAddr;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::commands::{
    ClientConnectedCommand, DataPacketClientCommand, LocalClientDisconnectedCommand, PingCommand,
};
use crate::{ClientState, ServerConfig};
use tcproxy_core::TcpFrame;
use tcproxy_core::{AsyncCommand, Result};
use crate::managers::IFeatureManager;

#[async_trait]
pub trait FrameHandler: Send + Sync {
    async fn handle(
        &mut self,
        frame: TcpFrame,
        cancellation_token: CancellationToken,
    ) -> Result<Option<TcpFrame>>;
}

pub struct DefaultFrameHandler {
    feature_manager: Arc<IFeatureManager>,
    sender: Sender<TcpFrame>,
    state: Arc<ClientState>,
}

impl DefaultFrameHandler {
    pub fn new(feature_manager: &Arc<IFeatureManager>, sender: &Sender<TcpFrame>, state: &Arc<ClientState>) -> Self {
        Self {
            feature_manager: feature_manager.clone(),
            sender: sender.clone(),
            state: state.clone(),
        }
    }
}

#[async_trait]
impl FrameHandler for DefaultFrameHandler {
    async fn handle(
        &mut self,
        frame: TcpFrame,
        cancellation_token: CancellationToken,
    ) -> Result<Option<TcpFrame>> {
        let mut command_handler: Box<dyn AsyncCommand<Output = Result<()>>> = match frame {
            TcpFrame::Ping => Box::new(PingCommand::new(&self.sender)),
            TcpFrame::LocalClientDisconnected { connection_id } => {
                LocalClientDisconnectedCommand::boxed_new(connection_id, &self.state)
            },
            TcpFrame::ClientPacket(data) => {
                DataPacketClientCommand::boxed_new(
                    data.buffer(),
                    data.connection_id(),
                    &self.state)
            }
            TcpFrame::ClientConnected => {
                let listen_ip = self.feature_manager
                    .get_config()
                    .get_listen_ip();

                ClientConnectedCommand::boxed_new(
                    listen_ip,
                    &self.sender,
                    &self.state,
                    &cancellation_token)
            },
            _ => {
                debug!("invalid frame received.");
                return Ok(None);
            }
        };

        command_handler.handle().await?;
        Ok(None)
    }
}
