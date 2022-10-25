use async_trait::async_trait;
use bytes::BytesMut;
use std::sync::Arc;
use tcproxy_core::{AsyncCommand, Result};
use uuid::Uuid;

use crate::ClientState;

pub struct DataPacketClientCommand {
    connection_id: Uuid,
    buffer: BytesMut,
    proxy_state: Arc<ClientState>,
}

impl DataPacketClientCommand {
    pub fn new(buffer: &BytesMut, connection_id: &Uuid, proxy_state: &Arc<ClientState>) -> Self {
        Self {
            buffer: buffer.clone(),
            connection_id: *connection_id,
            proxy_state: proxy_state.clone(),
        }
    }

    pub fn boxed_new(
        buffer: &BytesMut,
        connection_id: &Uuid,
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
        let (connection_sender, _) = match connection_manager.get_connection(self.connection_id)
        {
            Some(sender) => sender,
            None => return Ok(()),
        };

        let buffer = self.buffer.split();
        let _ = connection_sender.send(buffer).await;
        Ok(())
    }
}
