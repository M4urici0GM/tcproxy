use std::sync::Arc;

use async_trait::async_trait;
use tcproxy_core::AsyncCommand;
use tcproxy_core::Result;
use tracing::debug;
use uuid::Uuid;

use crate::ClientState;

/// issued when remote socket disconnects from server.
pub struct RemoteDisconnectedCommand {
    connection_id: Uuid,
    state: Arc<ClientState>,
}

impl RemoteDisconnectedCommand {
    pub fn new(connection_id: Uuid, state: &Arc<ClientState>) -> Self {
        Self {
            connection_id,
            state: state.clone(),
        }
    }
}

#[async_trait]
impl AsyncCommand for RemoteDisconnectedCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        let (sender, cancellation_token) = match self.state.remove_connection(self.connection_id) {
            Some(item) => item,
            None => {
                debug!("connection not found {}", self.connection_id);
                return Ok(());
            }
        };

        cancellation_token.cancel();
        drop(sender);
        Ok(())
    }
}
