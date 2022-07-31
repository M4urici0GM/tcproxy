use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use bytes::BytesMut;
use tokio::net::TcpStream;
use tokio::sync::{mpsc};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use std::sync::Mutex;
use uuid::Uuid;
use tracing::debug;

use crate::server::ProxyClientState;
use crate::{PortManager, Result};
use crate::codec::TcpFrame;
use crate::proxy::{ProxyClientStreamReader, ProxyClientStreamWriter};

#[derive(Debug)]
pub struct ProxyClient {
    pub(crate) listen_ip: Ipv4Addr,
    pub(crate) port_manager: PortManager,
    pub(crate) remote_ip: SocketAddr,
    pub(crate) proxy_state: Arc<ProxyClientState>,
}

impl ProxyClient {
    pub fn new(listen_ip: Ipv4Addr, remote_ip: SocketAddr, port_manager: PortManager, state: Arc<ProxyClientState>) -> Self {
        Self {
            listen_ip,
            port_manager,
            remote_ip,
            proxy_state: state,
        }
    }

    pub async fn start_streaming(&self, tcp_stream: TcpStream, cancellation_token: CancellationToken) -> Result<()> {
        let (reader, writer) = tcp_stream.into_split();
        let (main_sender, receiver) = mpsc::channel::<TcpFrame>(100);

        let client_cancellation_token = CancellationToken::new();
        let mut stream_reader = ProxyClientStreamReader {
            reader,
            target_ip:  self.listen_ip,
            remote_ip: self.remote_ip,
            proxy_client_sender: main_sender.clone(),
            connections: self.proxy_state.clone(),
            port_manager: self.port_manager.clone(),
            buffer: BytesMut::with_capacity(1024 * 8),
            cancellation_token: client_cancellation_token.child_token(),
        };

        let mut stream_writer = ProxyClientStreamWriter {
            writer,
            proxy_client_receiver: receiver,
            cancellation_token: client_cancellation_token.child_token(),
        };

        tokio::select! {
            res = stream_reader.read_from_socket() => {
                debug!("stream_reader.read_from_socket task finished.");
                debug!("{:?}", res);
            },
            _ = stream_writer.write_to_socket() => {
                debug!("stream_writer.write_to_socket task finished.");
            },
            _ = cancellation_token.cancelled() => {
                debug!("cancellation token cancelled.");
            }
        };

        client_cancellation_token.cancel();
        Ok(())
    }
}