use bytes::{Buf, BytesMut};
use futures_util::stream::SplitStream;
use tokio::io::AsyncReadExt;
use std::collections::HashMap;
use std::io::Cursor;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::codec::{TcpFrame, TcpFrameCodec};
use crate::proxy::ProxyServer;
use crate::PortManager;
use crate::Result;
use crate::server::ProxyClientState;

pub struct ProxyClientStreamReader {
    pub(crate) target_ip: Ipv4Addr,
    pub(crate) remote_ip: SocketAddr,
    pub(crate) port_manager: PortManager,
    pub(crate) proxy_client_sender: Sender<TcpFrame>,
    pub(crate) connections: Arc<ProxyClientState>,
    pub(crate) reader: OwnedReadHalf,
    pub(crate) buffer: BytesMut,
    pub(crate) cancellation_token: CancellationToken,
}

impl ProxyClientStreamReader {
    async fn parse_frame(&mut self) -> Result<Option<TcpFrame>> {
        let mut cursor = Cursor::new(&self.buffer[..]);
        match TcpFrame::check(&mut cursor) {
            Ok(_) => {
                let position = cursor.position() as usize;
                cursor.set_position(0);

                let frame = TcpFrame::parse(&mut cursor)?;
                self.buffer.advance(position);

                Ok(Some(frame))
            }
            Err(crate::codec::FrameError::Incomplete) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    async fn receive_frame(&mut self) -> Result<Option<TcpFrame>> {
        loop {
            if let Some(frame) = self.parse_frame().await? {
                return Ok(Some(frame));
            }

            if 0 == self.reader.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    debug!("received 0 bytes from client, and buffer is empty.");
                    return Ok(None);
                }

                return Err("connection reset by peer.".into());
            }
        }
    }

    async fn handle_frame(&mut self) -> Result<()> {
        loop {
            let frame = match self.receive_frame().await? {
                Some(f) => f,
                None => {
                    debug!("received none from client.");
                    return Ok(());
                },
            };

            match frame {
                TcpFrame::DataPacketClient {
                    buffer,
                    connection_id,
                } => {
                    let connection_sender = match self.connections.get_connection(connection_id) {
                        Some(c) => c.clone(),
                        None => {
                            continue;
                        }
                    };

                    match connection_sender.send(buffer).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!(
                                "failed when sending data to connection {}: {}",
                                connection_id, err
                            );
                            return Ok(());
                        }
                    };
                }
                TcpFrame::LocalClientDisconnected { connection_id } => {
                    debug!("connection {} was disconnected", connection_id);
                    self.connections.remove_connection(connection_id);                    
                },
                TcpFrame::Ping => match self.proxy_client_sender.send(TcpFrame::Pong).await {
                    Ok(_) => debug!("Sent Pong to client."),
                    Err(err) => error!("Failed to send ping back to client. {}", err),
                },
                TcpFrame::ClientConnected => {
                    let listen_ip = self.target_ip;
                    let port = self.port_manager.get_port().await?;

                    let host_sender = self.proxy_client_sender.clone();
                    let cancellation_token = self.cancellation_token.child_token();

                    match host_sender
                        .send(TcpFrame::ClientConnectedAck { port })
                        .await
                    {
                        Ok(_) => {
                            info!("Successfully send ACK package to {}", self.remote_ip);
                        }
                        Err(err) => {
                            error!(
                                "Failed when sending ACK package to {}: {}",
                                self.remote_ip, err
                            );
                            return Err("closing connection due invalid sender.".into());
                        }
                    };

                    debug!("spawning new proxy server..");
                    let state_clone = self.connections.clone();
                    tokio::spawn(async move {
                        let proxy_server = ProxyServer {
                            host_sender,
                            available_connections: state_clone,
                            listen_ip,
                            port,
                        };

                        tokio::select! {
                            _ = proxy_server.listen() => {
                                debug!("PROXY SERVER TASK FINISHED");
                            },
                            _ = cancellation_token.cancelled() => {
                                info!("received cancellation signal");
                                info!("client disconnected. closing server {}:{}...", listen_ip, port);
                            }
                        }
                    });
                }
                _ => {}
            }
        }
    }

    pub async fn read_from_socket(&mut self) -> Result<()> {
        let token = self.cancellation_token.child_token();
        tokio::select! {
            _ = self.handle_frame() => {},
            _ = token.cancelled() => {},
        };

        Ok(())
    }
}
