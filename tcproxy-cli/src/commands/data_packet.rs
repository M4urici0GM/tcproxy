use std::sync::Arc;

use async_trait::async_trait;
use bytes::BytesMut;
use tcproxy_core::{Command, Result};
use tracing::debug;
use uuid::Uuid;

use crate::ClientState;

/// issued when server receives new data packet.
pub struct DataPacketCommand {
    connection_id: Uuid,
    buffer: BytesMut,
    buffer_size: u32,
    state: Arc<ClientState>,
}

impl DataPacketCommand {
    pub fn new(
        connection_id: Uuid,
        buffer: BytesMut,
        buffer_size: u32,
        state: &Arc<ClientState>,
    ) -> Self {
        Self {
            connection_id,
            buffer,
            buffer_size,
            state: state.clone(),
        }
    }
}

#[async_trait]
impl Command for DataPacketCommand {
    type Output = ();

    async fn handle(&mut self) -> Result<()> {
        debug!("received new packet from {}", self.connection_id);
        match self.state.get_connection(self.connection_id) {
            Some((sender, _)) => {
                let sender_clone = sender.clone();
                let buffer = BytesMut::from(&self.buffer[..self.buffer_size as usize]);
                let _ = sender_clone.send(buffer).await;
            }
            None => {
                debug!("connection {} not found!", self.connection_id);
            }
        };

        Ok(())
    }
}
