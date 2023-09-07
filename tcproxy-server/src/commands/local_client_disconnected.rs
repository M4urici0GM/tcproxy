use async_trait::async_trait;
use std::sync::Arc;
use tcproxy_core::{framing::SocketDisconnected, TcpFrame};
use tokio::sync::mpsc::Sender;

use crate::ClientState;

use super::NewFrameHandler;

pub struct SocketDisconnectedHandler(tcproxy_core::framing::SocketDisconnected);

impl From<SocketDisconnected> for SocketDisconnectedHandler {
    fn from(value: SocketDisconnected) -> Self {
        Self(value)
    }
}

impl Into<Box<dyn NewFrameHandler>> for SocketDisconnectedHandler {
    fn into(self) -> Box<dyn NewFrameHandler> {
        Box::new(self)
    }
}

#[async_trait]
impl NewFrameHandler for SocketDisconnectedHandler {
    async fn execute(
        &self,
        _tx: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
    ) -> tcproxy_core::Result<Option<TcpFrame>> {
        let connection_id = self.0.connection_id();
        tracing::debug!("connection {} disconnected from client", connection_id);

        match state
            .get_connection_manager()
            .remove_connection(&connection_id)
        {
            Some((_, token)) => {
                token.cancel();
                tracing::debug!("cancelled task for connection {}", connection_id);
            }
            None => {
                tracing::warn!(
                    "connection {} not found on connection state.",
                    connection_id
                );
            }
        }

        tracing::debug!("removed connection {} from connection state", connection_id);
        Ok(None)
    }
}
