use async_trait::async_trait;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::commands::{
    ClientConnectedCommand, DataPacketClientCommand, LocalClientDisconnectedCommand, PingCommand,
};
use crate::ClientState;
use tcproxy_core::TcpFrame;
use tcproxy_core::{AsyncCommand, Result};
use tcproxy_core::TcpFrame::ClientConnected;

#[async_trait]
pub trait FrameHandler: Sync + Send {
    async fn handle_frame(
        &mut self,
        frame: TcpFrame,
        cancellation_token: CancellationToken,
    ) -> Result<Option<TcpFrame>>;
}

pub struct DefaultFrameHandler {
    target_ip: IpAddr,
    frame_tx: Sender<TcpFrame>,
    state: Arc<ClientState>,
}

impl DefaultFrameHandler {
    pub fn new(ip: &IpAddr, sender: &Sender<TcpFrame>, state: &Arc<ClientState>) -> Self {
        Self {
            target_ip: *ip,
            frame_tx: sender.clone(),
            state: state.clone(),
        }
    }
}

#[async_trait]
impl FrameHandler for DefaultFrameHandler {
    async fn handle_frame(
        &mut self,
        frame: TcpFrame,
        cancellation_token: CancellationToken,
    ) -> Result<Option<TcpFrame>> {
        let mut command_handler: Box<dyn AsyncCommand<Output = Result<()>>> = match frame {
            TcpFrame::Ping => Box::new(PingCommand::new(&self.frame_tx)),
            TcpFrame::LocalClientDisconnected { connection_id } => Box::new(
                LocalClientDisconnectedCommand::new(connection_id, &self.state),
            ),
            TcpFrame::ClientPacket(data) => {
                DataPacketClientCommand::boxed_new(data.buffer(), data.connection_id(), &self.state)
            }
            TcpFrame::ClientConnected => Box::new(ClientConnectedCommand::new(
                    &self.target_ip,
                &self.frame_tx,
                &self.state,
                &cancellation_token,
            )),
            _ => {
                debug!("invalid frame received.");
                return Ok(None);
            }
        };

        command_handler.handle().await?;
        Ok(None)
    }
}
