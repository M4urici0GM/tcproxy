use async_trait::async_trait;
use bytes::BytesMut;
use tcproxy_core::Result;
use std::sync::Arc;
use uuid::Uuid;
use tcproxy_core::Command;

use crate::ProxyState;

pub struct DataPacketClientCommand {
    connection_id: Uuid,
    buffer: BytesMut,
    proxy_state: Arc<ProxyState>,
}

impl DataPacketClientCommand {
    pub fn new(buffer: BytesMut, connection_id: &Uuid, proxy_state: &Arc<ProxyState>) -> Self {
        Self {
            buffer,
            connection_id: connection_id.clone(),
            proxy_state: proxy_state.clone(),
        }
    }
}

#[async_trait]
impl Command for DataPacketClientCommand {
    async fn handle(&mut self) -> Result<()> {
        let (connection_sender, _) = match self.proxy_state.connections.get_connection(self.connection_id) {
            Some(sender) => sender,
            None => return Ok(()),
        };

        let buffer = self.buffer.split();
        let _ = connection_sender.send(buffer).await;
        Ok(())
    }
}
