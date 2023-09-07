use std::sync::Arc;

use async_trait::async_trait;
use tcproxy_core::framing::Pong;
use tcproxy_core::TcpFrame;
use tokio::sync::mpsc::Sender;

use crate::ClientState;

use super::NewFrameHandler;

pub struct PingFrameHandler(tcproxy_core::framing::Ping);

#[async_trait]
impl NewFrameHandler for PingFrameHandler {
    async fn execute(
        &self,
        tx: &Sender<TcpFrame>,
        _state: &Arc<ClientState>,
    ) -> tcproxy_core::Result<Option<TcpFrame>> {
        tx.send(TcpFrame::Pong(Pong::new())).await?;

        Ok(None)
    }
}

impl From<tcproxy_core::framing::Ping> for PingFrameHandler {
    fn from(value: tcproxy_core::framing::Ping) -> Self {
        Self(value)
    }
}

impl From<PingFrameHandler> for Box<dyn NewFrameHandler> {
    fn from(val: PingFrameHandler) -> Self {
        Box::new(val)
    }
}
