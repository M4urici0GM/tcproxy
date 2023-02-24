use std::cell::{Cell, RefCell};
use async_trait::async_trait;


use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::commands::{AuthenticateArgs, AuthenticateCommand, ClientConnectedCommand, DataPacketClientCommand, LocalClientDisconnectedCommand, PingCommand};
use crate::{ClientState};
use tcproxy_core::TcpFrame;
use tcproxy_core::{AsyncCommand, Result};
use tcproxy_core::auth::token_handler::TokenHandler;


#[async_trait]
pub trait FrameHandler: Send + Sync {
    async fn handle(
        &mut self,
        frame: TcpFrame,
        cancellation_token: CancellationToken,
    ) -> Result<Option<TcpFrame>>;
}

pub struct DefaultFrameHandler {
    sender: Sender<TcpFrame>,
    state: Arc<ClientState>,
    token_handler: Arc<Box<dyn TokenHandler + 'static>>,
}

impl DefaultFrameHandler {
    pub fn new<T>(
        sender: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
        token_handler: T,
    ) -> Self
        where T: TokenHandler + 'static
    {
        Self {
            sender: sender.clone(),
            state: state.clone(),
            token_handler: Arc::new(Box::new(token_handler)),
        }
    }
}

#[async_trait]
impl FrameHandler for DefaultFrameHandler {
    async fn handle(
        &mut self,
        frame: TcpFrame,
        _cancellation_token: CancellationToken,
    ) -> Result<Option<TcpFrame>> {
        let mut command_handler: Box<dyn AsyncCommand<Output=Result<()>>> = match frame {
            TcpFrame::Ping(_) => Box::new(PingCommand::new(&self.sender)),
            TcpFrame::SocketDisconnected(data) => {
                LocalClientDisconnectedCommand::boxed_new(data.connection_id(), &self.state)
            }
            TcpFrame::DataPacket(data) => {
                DataPacketClientCommand::boxed_new(
                    data.buffer(),
                    data.connection_id(),
                    &self.state)
            }
            TcpFrame::ClientConnected(_) => ClientConnectedCommand::boxed_new(&self.sender),
            TcpFrame::Authenticate(data) => AuthenticateCommand::boxed_new(
                AuthenticateArgs::from(data),
                &self.sender,
                self.state.get_auth_manager(),
                &self.token_handler,
                &self.state.get_accounts_manager()),
            _ => {
                debug!("invalid frame received.");
                return Ok(None);
            }
        };

        command_handler.handle().await?;
        Ok(None)
    }
}

