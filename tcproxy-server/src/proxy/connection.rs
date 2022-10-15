use std::net::IpAddr;
use std::ops::Range;
use std::sync::Arc;
use tcproxy_core::tcp::SocketConnection;
use tcproxy_core::transport::TcpFrameTransport;
use tcproxy_core::{Result, TcpFrame};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::proxy::{ClientFrameReader, ClientFrameWriter};
use crate::ClientState;

#[derive(Debug)]
pub struct ClientConnection {
    pub(crate) listen_ip: IpAddr,
    pub(crate) state: Arc<ClientState>,
}

impl ClientConnection {
    pub fn new(listen_ip: &IpAddr, port_range: &Range<u16>) -> Self {
        Self {
            listen_ip: listen_ip.clone(),
            state: ClientState::new(&port_range),
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
        let transport = TcpFrameTransport::new(tcp_stream);
        let local_cancellation_token = CancellationToken::new();

        let (reader, writer) = transport.split();
        let (frame_tx, frame_rx) = mpsc::channel::<TcpFrame>(10000);

        let client_reader = ClientFrameReader {
            reader,
            frame_tx,
            target_ip: self.listen_ip,
            state: self.state.clone(),
        };

        let proxy_writer = ClientFrameWriter {
            writer,
            receiver: frame_rx,
            cancellation_token: local_cancellation_token.child_token(),
        };

        tokio::select! {
            res = proxy_writer.start_writing() => {
                debug!("ProxyClientStreamWriter::start_writing task completed with {:?}", res)
            },
            res = client_reader.start_reading(local_cancellation_token.child_token()) => {
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
