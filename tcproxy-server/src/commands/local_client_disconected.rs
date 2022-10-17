use async_trait::async_trait;
use std::sync::Arc;
use tcproxy_core::{AsyncCommand, Result};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::ClientState;

pub struct LocalClientDisconnectedCommand {
    connection_id: Uuid,
    proxy_state: Arc<ClientState>,
}

impl LocalClientDisconnectedCommand {
    pub fn new(connection_id: Uuid, proxy_state: &Arc<ClientState>) -> Self {
        Self {
            connection_id,
            proxy_state: proxy_state.clone(),
        }
    }
}

#[async_trait]
impl AsyncCommand for LocalClientDisconnectedCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        debug!("connection {} disconnected from client", self.connection_id);
        let result = self
            .proxy_state
            .connections
            .remove_connection(self.connection_id);

        match result {
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
