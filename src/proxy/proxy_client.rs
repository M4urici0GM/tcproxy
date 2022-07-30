use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc};
use bytes::BytesMut;
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{PortManager, Result};
use crate::codec::{TcpFrameCodec, TcpFrame};
use crate::proxy::{ProxyClientStreamReader, ProxyClientStreamWriter};

#[derive(Debug)]
pub struct ProxyClient {
    pub(crate) listen_ip: Ipv4Addr,
    pub(crate) port_manager: PortManager,
    pub(crate) remote_ip: SocketAddr,
}

impl ProxyClient {
    pub fn new(listen_ip: Ipv4Addr, remote_ip: SocketAddr, port_manager: PortManager) -> Self {
        Self {
            listen_ip,
            port_manager,
            remote_ip,
        }
    }

    pub async fn start_streaming(&self, tcp_stream: TcpStream, cancellation_token: CancellationToken) -> Result<()> {
        let transport = Framed::new(tcp_stream, TcpFrameCodec);
        let (transport_writer, transport_reader) = transport.split();

        let connections: Arc<Mutex<HashMap<Uuid, Sender<BytesMut>>>> = Arc::new(Mutex::new(HashMap::new()));
        let (main_sender, receiver) = mpsc::channel::<TcpFrame>(100);

        let client_cancellation_token = CancellationToken::new();
        let mut stream_reader = ProxyClientStreamReader {
            target_ip:  self.listen_ip,
            proxy_client_sender: main_sender.clone(),
            reader: transport_reader,
            connections: connections.clone(),
            port_manager: self.port_manager.clone(),
            remote_ip: self.remote_ip,
        };

        let mut stream_writer = ProxyClientStreamWriter {
            proxy_client_receiver: receiver,
            writer: transport_writer,
            remote_ip: self.remote_ip,
        };

        tokio::select! {
            _ = stream_reader.read_from_socket(client_cancellation_token.child_token()) => {},
            _ = stream_writer.write_to_socket(client_cancellation_token.child_token()) => {},
            _ = cancellation_token.cancelled() => {}
        };

        client_cancellation_token.cancel();
        Ok(())
    }
}