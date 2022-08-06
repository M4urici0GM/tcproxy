use std::net::Ipv4Addr;
use std::sync::Arc;
use tcproxy_core::{Result, TcpFrame};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::proxy::{ProxyClientStreamReader, ProxyClientStreamWriter};
use crate::ProxyState;


#[derive(Debug)]
pub struct Connection {
    pub(crate) listen_ip: Ipv4Addr,
    pub(crate) state: Arc<ProxyState>,
}

impl Connection {
    pub fn new(listen_ip: Ipv4Addr, state: Arc<ProxyState>) -> Self {
        Self {
            listen_ip,
            state,
        }
    }

    pub async fn start_streaming(&mut self, tcp_stream: TcpStream, cancellation_token: CancellationToken) -> Result<()> {
        let (connection_reader, connection_writer) = tcp_stream.into_split();
        let local_cancellation_token = CancellationToken::new();

        let (client_sender, client_reader) = mpsc::channel::<TcpFrame>(10000);
        let mut proxy_reader = ProxyClientStreamReader {
            target_ip: self.listen_ip,
            sender: client_sender.clone(),
            state: self.state.clone(),
        };

        let mut proxy_writer = ProxyClientStreamWriter {
            receiver: client_reader,
            writer: connection_writer,
            cancellation_token: local_cancellation_token.clone(),
        };

        tokio::select! {
            res = proxy_reader.start_reading(connection_reader, local_cancellation_token.clone()) => {
                debug!("ProxyClient::create_frame_reader task completed with {:?}", res);
            },
            res = proxy_writer.start_writing() => {
                debug!("ProxyClient::create_frame_writer task completed with {:?}", res)
            },
            _ = cancellation_token.cancelled() => {
                debug!("received global stop signal..");
            },
        };

        local_cancellation_token.cancel();
        Ok(())
    }
}
