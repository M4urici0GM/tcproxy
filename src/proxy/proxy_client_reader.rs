use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use bytes::BytesMut;
use futures_util::stream::SplitStream;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use tracing::{error, debug, info};

use crate::Result;
use crate::codec::{TcpFrame, TcpFrameCodec};
use crate::PortManager;
use crate::proxy::ProxyServer;

pub struct ProxyClientStreamReader {
    pub(crate) target_ip: Ipv4Addr,
    pub(crate) port_manager: PortManager,
    pub(crate) proxy_client_sender: Sender<TcpFrame>,
    pub(crate) connections: Arc<Mutex<HashMap<Uuid, Sender<BytesMut>>>>,
    pub(crate) reader: SplitStream<Framed<TcpStream, TcpFrameCodec>>,
}

impl ProxyClientStreamReader {
    async fn receive_frame(&mut self) -> Result<TcpFrame> {
        let received = self.reader.next().await;
        if received.is_none() {
            debug!("No frame received from client. Aborting.");
            return Err("No frame received from client. aborting".into());
        }

        let frame = match received.unwrap() {
            Ok(frame) => frame,
            Err(err) => {
                error!("Error when parsing frame. {}", err);
                return Err(err.into());
            }
        };

        Ok(frame)
    }

    pub async fn read_from_socket(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        while !cancellation_token.is_cancelled() {
            let frame = self.receive_frame().await?;

            match frame {
                TcpFrame::DataPacket { buffer, connection_id } => {
                    let connections_lock = self.connections.lock().await;

                    if !connections_lock.contains_key(&connection_id) {
                        error!("connection id {} not found.", connection_id);
                        return Ok(());
                    }

                    let connection_sender = connections_lock.get(&connection_id).unwrap();
                    match connection_sender.send(buffer).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("failed when sending data to connection {}: {}", connection_id, err);
                            return Ok(());
                        }
                    };
                }
                TcpFrame::ClientConnected => {
                    let listen_ip = self.target_ip;
                    let port = self.port_manager.get_port().await?;

                    let connections = self.connections.clone();
                    let host_sender = self.proxy_client_sender.clone();
                    let cancellation_token = cancellation_token.child_token();

                    tokio::spawn(async move {
                        let proxy_server = ProxyServer {
                            host_sender,
                            available_connections: connections,
                            listen_ip,
                            port,
                        };

                        tokio::select! {
                            _ = proxy_server.listen() => {},
                            _ = cancellation_token.cancelled() => {
                                info!("client disconnected. closing server {}:{}...", listen_ip, port);
                            }
                        }
                    });
                }
                _ => {}
            }
        };

        Ok(())
    }
}