use std::sync::Arc;
use tcproxy_core::tcp::SocketConnection;
use tcproxy_core::transport::TcpFrameTransport;
use tcproxy_core::{Result, TcpFrame};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::proxy::{DefaultFrameHandler, DefaultTokenHandler};
use crate::proxy::{ClientFrameReader, ClientFrameWriter};
use crate::{ClientState, ServerConfig};
use crate::managers::{UserManager, AuthenticationManagerGuard, IFeatureManager, PortManagerGuard};

pub struct ClientConnection {
    state: Arc<ClientState>,
    server_config: Arc<ServerConfig>,
}

impl ClientConnection {
    pub fn new(
        port_guard: Arc<PortManagerGuard>,
        auth_guard: Arc<AuthenticationManagerGuard>,
        server_config: &Arc<ServerConfig>,
        account_manager: &Arc<Box<dyn UserManager + 'static>>
    ) -> Self
    {
        Self {
            state: ClientState::new(port_guard, auth_guard, server_config, account_manager),
            server_config: server_config.clone(),
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

        let token_handler = DefaultTokenHandler::new(&self.server_config);
        let frame_handler = DefaultFrameHandler::new(&frame_tx, &self.state, token_handler);
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
        }
        ;

        local_cancellation_token.cancel();
        Ok(())
    }
}
