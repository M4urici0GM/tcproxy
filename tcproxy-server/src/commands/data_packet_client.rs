use async_trait::async_trait;
use std::sync::Arc;
use tcproxy_core::{AsyncCommand, Result};

use crate::ClientState;

pub struct DataPacketClientCommand {
    connection_id: u32,
    buffer: Vec<u8>,
    proxy_state: Arc<ClientState>,
}

impl DataPacketClientCommand {
    pub fn new(buffer: &[u8], connection_id: &u32, proxy_state: &Arc<ClientState>) -> Self {
        Self {
            buffer: buffer.to_vec(),
            connection_id: *connection_id,
            proxy_state: proxy_state.clone(),
        }
    }

    pub fn boxed_new(
        buffer: &[u8],
        connection_id: &u32,
        proxy_state: &Arc<ClientState>,
    ) -> Box<Self> {
        let obj = DataPacketClientCommand::new(buffer, connection_id, proxy_state);
        Box::new(obj)
    }
}

#[async_trait]
impl AsyncCommand for DataPacketClientCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        let connection_manager = self.proxy_state.get_connection_manager();
        let (connection_sender, _) = match connection_manager.get_connection(&self.connection_id)
        {
            Some(sender) => sender,
            None => return Ok(()),
        };

        let _ = connection_sender.send(self.buffer.clone()).await;
        Ok(())
    }
}
