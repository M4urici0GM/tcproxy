use std::sync::Arc;

use async_trait::async_trait;
use bytes::BytesMut;
use tracing::debug;
use tcproxy_core::{Command, Result};
use uuid::Uuid;

use crate::ClientState;

pub struct DataPacketCommand {
  connection_id: Uuid,
  buffer: BytesMut,
  state: Arc<ClientState>,
}

impl DataPacketCommand {
  pub fn new(connection_id: Uuid, buffer: BytesMut, state: &Arc<ClientState>) -> Self {
    Self {
      connection_id,
      buffer,
      state: state.clone(),
    }
  }
}

#[async_trait]
impl Command for DataPacketCommand {
    async fn handle(&self) -> Result<()> {
        debug!("received new packet from {}", self.connection_id);
        match self.state.get_connection(self.connection_id) {
            Some((sender, _)) => {
                let sender_clone = sender.clone();
                let buffer = BytesMut::from(&self.buffer[..]);
                let _ = sender_clone.send(buffer).await;
            }
            None => {
                debug!("connection {} not found!", self.connection_id);
            }
        };

        Ok(())
    }
}
