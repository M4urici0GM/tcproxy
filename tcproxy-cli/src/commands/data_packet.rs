use std::sync::Arc;

use async_trait::async_trait;
use bytes::BytesMut;
use tcproxy_core::{AsyncCommand, Result};
use tracing::debug;

use crate::ClientState;

/// issued when server receives new data packet.
pub struct DataPacketCommand {
    connection_id: u32,
    buffer: Vec<u8>,
    state: Arc<ClientState>,
}

impl DataPacketCommand {
    pub fn new(connection_id: &u32, buffer: &[u8], state: &Arc<ClientState>) -> Self {
        Self {
            connection_id: *connection_id,
            buffer: buffer.to_vec(),
            state: state.clone(),
        }
    }

    pub fn connection_id(&self) -> &u32 {
        &self.connection_id
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
}

#[async_trait]
impl AsyncCommand for DataPacketCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        debug!("received new packet from {}", self.connection_id);
        match self.state.get_connection(&self.connection_id) {
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
