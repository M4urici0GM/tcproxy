use std::net::Ipv4Addr;
use std::sync::Arc;
use tcproxy_core::{Result, TcpFrame, TcpFrameTransport};
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
        let transport = TcpFrameTransport::new(tcp_stream);
        let local_cancellation_token = CancellationToken::new();

        let (reader, writer) = transport.split();
        let (client_sender, client_reader) = mpsc::channel::<TcpFrame>(10000);
        let proxy_reader = ProxyClientStreamReader {
            target_ip: self.listen_ip,
            sender: client_sender.clone(),
            state: self.state.clone(),
            reader,
        };

        let mut proxy_writer = ProxyClientStreamWriter {
            writer,
            receiver: client_reader,
            cancellation_token: local_cancellation_token.clone(),
        };

        tokio::select! {
            res = proxy_writer.start_writing() => {
                debug!("ProxyClientStreamWriter::start_writing task completed with {:?}", res)
            },
            res = proxy_reader.start_reading(local_cancellation_token.clone()) => {
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
