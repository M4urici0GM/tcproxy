use async_trait::async_trait;
use std::sync::Arc;
use tcproxy_core::{framing::DataPacket, Result, TcpFrame};
use tokio::sync::mpsc::Sender;

use crate::ClientState;

use super::NewFrameHandler;

pub struct DataPacketHandler(DataPacket);

impl DataPacketHandler {
    fn get_buffer(&self) -> &[u8] {
        self.0.buffer()
    }
}

impl From<DataPacket> for DataPacketHandler {
    fn from(value: DataPacket) -> Self {
        Self(value)
    }
}

impl From<DataPacketHandler> for Box<dyn NewFrameHandler> {
    fn from(val: DataPacketHandler) -> Self {
        Box::new(val)
    }
}

#[async_trait]
impl NewFrameHandler for DataPacketHandler {
    async fn execute(
        &self,
        _tx: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
    ) -> Result<Option<TcpFrame>> {
        let connection_id = self.0.connection_id();
        let connection_manager = state.get_connection_manager();
        let (connection_sender, _) = match connection_manager.get_connection(connection_id) {
            Some(sender) => sender,
            None => return Ok(None),
        };

        match connection_sender.send(self.get_buffer().into()).await {
            Ok(_) => {}
            Err(err) => tracing::warn!(
                "failed when sending buffer to connection {}: {}",
                connection_id,
                err
            ),
        }

        Ok(None)
    }
}
