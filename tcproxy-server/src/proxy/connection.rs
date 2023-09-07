use std::sync::Arc;
use tcproxy_core::stream::Stream;
use tcproxy_core::transport::TcpFrameTransport;
use tcproxy_core::{Result, TcpFrame};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::managers::{AuthenticationManagerGuard, PortManager, UserManager};
use crate::proxy::{ClientFrameReader, ClientFrameWriter};
use crate::{ClientState, ServerConfig};

pub struct ClientConnection {
    state: Arc<ClientState>,
}

impl ClientConnection {
    pub fn new(
        port_guard: PortManager,
        auth_guard: Arc<AuthenticationManagerGuard>,
        server_config: &Arc<ServerConfig>,
        account_manager: &Arc<impl UserManager + 'static>,
    ) -> Self {
        Self {
            state: ClientState::new(port_guard, auth_guard, server_config, account_manager),
        }
    }

    /// Starts reading and writing to client.
    pub async fn start_streaming(
        &mut self,
        stream: Stream,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        let local_cancellation_token = CancellationToken::new();
        let (transport_reader, transport_writer) = TcpFrameTransport::new(stream).split();
        let (frame_tx, frame_rx) = mpsc::channel::<TcpFrame>(10000);
        let client_reader = ClientFrameReader::new(transport_reader, &self.state, &frame_tx);
        let proxy_writer =
            ClientFrameWriter::new(frame_rx, transport_writer, &local_cancellation_token);

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
