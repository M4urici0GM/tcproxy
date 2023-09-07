mod client_connected;
mod data_packet_client;
mod local_client_disconnected;
mod ping;

use std::sync::Arc;

use async_trait::async_trait;
pub use client_connected::*;
pub use data_packet_client::*;
pub use local_client_disconnected::*;
pub use ping::*;
use tcproxy_core::TcpFrame;
use tokio::sync::mpsc::Sender;

use crate::ClientState;

pub mod authenticate;

#[async_trait]
pub trait NewFrameHandler: Send + Sync {
    async fn execute(
        &self,
        tx: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
    ) -> tcproxy_core::Result<Option<TcpFrame>>;
}
