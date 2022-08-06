use std::sync::Arc;
use tracing::{debug, warn};
use async_trait::async_trait;
use uuid::Uuid;
use tcproxy_core::{Result, Command};

use crate::ProxyState;

pub struct LocalClientDisconnectedCommand {
  connection_id: Uuid,
  proxy_state: Arc<ProxyState>,
}

impl LocalClientDisconnectedCommand {
  pub fn new(connection_id: Uuid, proxy_state: &Arc<ProxyState>) -> Self {
    Self {
      connection_id,
      proxy_state: proxy_state.clone(),
    }
  }
}

#[async_trait]
impl Command for LocalClientDisconnectedCommand {
    async fn handle(&mut self)  -> Result<()> {
      debug!("connection {} disconnected from client", self.connection_id);
      match self.proxy_state.connections.remove_connection(self.connection_id) {
          Some((_, token)) => {
              token.cancel();
              debug!("cancelled task for connection {}", self.connection_id);
          },
          None => {
              warn!("connection {} not found on connection state.", self.connection_id);
          },
      }

      debug!("removed connection {} from connection state", self.connection_id);
      Ok(())
    }
}