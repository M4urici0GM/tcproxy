use bytes::BytesMut;
use futures_util::StreamExt;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{self, Sender};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::codec::TcpFrame;
use crate::server::ProxyClientState;
use crate::tcp::ListenerUtils;
use crate::Result;

pub struct ProxyServer {
    pub(crate) listen_ip: Ipv4Addr,
    pub(crate) port: u16,
    pub(crate) host_sender: Sender<TcpFrame>,
    pub(crate) available_connections: Arc<ProxyClientState>,
}

impl ProxyServer {
    pub async fn listen(&self) -> Result<()> {
        let listener = ListenerUtils {
            port: self.port,
            ip: self.listen_ip,
        };
        let tcp_socket = listener.bind().await.unwrap();

        loop {
            let (connection, _) = tcp_socket.accept().await?;
            let host_sender = self.host_sender.clone();
            let available_connections = self.available_connections.clone();
            let (mut reader, mut writer) = connection.into_split();
            let connection_id = Uuid::new_v4();

            let (sender, mut receiver) = mpsc::channel::<BytesMut>(100);

            available_connections.insert_connection(connection_id, sender.clone());

            info!(
                "received new socket in listener {}",
                ListenerUtils::create_socket_ip(self.listen_ip, self.port)
            );
            tokio::spawn(async move {
                let mut rx = Box::pin(async_stream::stream! {
                    let mut buffer = BytesMut::with_capacity(1024 * 8);
                    loop {
                        let bytes_read = match reader.read_buf(&mut buffer).await {
                            Ok(size) => size,
                            Err(err) => {
                                error!("Failed when reading from connection {}: {}", connection_id, err);
                                return;
                            }
                        };

                        if 0 == bytes_read {
                            debug!("reached end of stream");
                            return;
                        }

                        buffer.truncate(bytes_read);

                        yield TcpFrame::DataPacketHost { connection_id, buffer: buffer.clone() };
                    }
                });

                let sender_clone = host_sender.clone();
                let _ = host_sender
                    .send(TcpFrame::IncomingSocket { connection_id })
                    .await;

                let task1 = tokio::spawn(async move {
                    loop {
                        let msg = receiver.recv().await;
                        if msg.is_none() {
                            break;
                        }

                        let _ = writer.write_buf(&mut msg.unwrap()).await;
                    }
                });

                let task2 = tokio::spawn(async move {
                    loop {
                        let frame = rx.next().await;
                        if frame.is_none() {
                            break;
                        }

                        match host_sender.send(frame.unwrap()).await {
                            Ok(_) => {}
                            Err(err) => {
                                debug!("error sending stuff to host_sender {}", err);
                                break;
                            }
                        };
                    }
                });

                tokio::select! {
                    _ = task1 => {},
                    _ = task2 => {},
                };

                let _ = sender_clone
                    .send(TcpFrame::RemoteSocketDisconnected { connection_id })
                    .await;
            });
        }
    }
}
