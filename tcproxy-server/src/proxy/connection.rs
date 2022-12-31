


use std::sync::Arc;
use tcproxy_core::tcp::SocketConnection;
use tcproxy_core::transport::TcpFrameTransport;
use tcproxy_core::{Result, TcpFrame};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::proxy::DefaultFrameHandler;
use crate::proxy::{ClientFrameReader, ClientFrameWriter};
use crate::{ClientState};
use crate::managers::IFeatureManager;

pub struct ClientConnection {
    server_config: Arc<IFeatureManager>,
    state: Arc<ClientState>,
}

impl ClientConnection {
    pub fn new(feature_manager: &Arc<IFeatureManager>) -> Self {
        Self {
            server_config: feature_manager.clone(),
            state: ClientState::new(feature_manager),
        }
    }

    /// Starts reading and writing to client.
    pub async fn start_streaming<T>(
        &mut self,
        tcp_stream: T,
        cancellation_token: CancellationToken,
    ) -> Result<()>
    where
        T: SocketConnection,
    {
        let local_cancellation_token = CancellationToken::new();
        let (transport_reader, transport_writer) = TcpFrameTransport::new(tcp_stream).split();

        let (frame_tx, frame_rx) = mpsc::channel::<TcpFrame>(10000);

        let frame_handler = DefaultFrameHandler::new(&self.server_config, &frame_tx, &self.state);
        let client_reader = ClientFrameReader::new(&frame_tx, transport_reader, frame_handler);
        let proxy_writer = ClientFrameWriter::new(frame_rx, transport_writer, &local_cancellation_token);

        tokio::select! {
            res = proxy_writer.spawn() => {
                debug!("ProxyClientStreamWriter::start_writing task completed with {:?}", res)
            },
            res = client_reader.spawn(local_cancellation_token.child_token()) => {
                debug!("ProxyClientStreamWriter::start_reading task completed with {:?}", res);
            },
            _ = cancellation_token.cancelled() => {
                debug!("received global stop signal..");
            },
        };

        local_cancellation_token.cancel();
        Ok(())
    }
}
