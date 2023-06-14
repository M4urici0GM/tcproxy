use async_trait::async_trait;
use std::sync::Arc;
use tcproxy_core::{AsyncCommand, Result};
use tracing::{debug, warn};

use crate::ClientState;

pub struct LocalClientDisconnectedCommand {
    connection_id: u32,
    proxy_state: Arc<ClientState>,
}

impl LocalClientDisconnectedCommand {
    pub fn new(connection_id: &u32, proxy_state: &Arc<ClientState>) -> Self {
        Self {
            connection_id: *connection_id,
            proxy_state: proxy_state.clone(),
        }
    }

    pub fn boxed_new(connection_id: &u32, state: &Arc<ClientState>) -> Box<Self> {
        let local_self = LocalClientDisconnectedCommand::new(connection_id, state);
        Box::new(local_self)
    }
}

#[async_trait]
impl AsyncCommand for LocalClientDisconnectedCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        debug!("connection {} disconnected from client", self.connection_id);

        match self
            .proxy_state
            .get_connection_manager()
            .remove_connection(&self.connection_id)
        {
            Some((_, token)) => {
                token.cancel();
                debug!("cancelled task for connection {}", self.connection_id);
            }
            None => {
                warn!(
                    "connection {} not found on connection state.",
                    self.connection_id
                );
            }
        }

        debug!(
            "removed connection {} from connection state",
            self.connection_id
        );
        Ok(())
    }
}
