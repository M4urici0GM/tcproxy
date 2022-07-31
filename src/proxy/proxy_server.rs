use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::{Arc};
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Mutex;
use futures_util::StreamExt;
use uuid::Uuid;
use tracing::{info, error, debug};

use crate::Result;
use crate::codec::TcpFrame;
use crate::tcp::Listener;

pub struct ProxyServer {
    pub(crate) listen_ip: Ipv4Addr,
    pub(crate) port: u16,
    pub(crate) host_sender: Sender<TcpFrame>,
    pub(crate) available_connections: Arc<Mutex<HashMap<Uuid, Sender<BytesMut>>>>,
}

impl ProxyServer {
    pub async fn listen(&self) -> Result<()> {
        let listener = Listener { port: self.port, ip: self.listen_ip };
        let tcp_socket = listener.bind().await.unwrap();

        loop {
            let (mut connection, _) = tcp_socket.accept().await?;
            let host_sender = self.host_sender.clone();
            let available_connections = self.available_connections.clone();

            info!("received new socket in listener {}", Listener::create_socket_ip(self.listen_ip, self.port));
            tokio::spawn(async move {
                let connection_id = Uuid::new_v4(); 
                let (mut reader, mut writer) = connection.split();

                let (sender, mut receiver) = mpsc::channel::<BytesMut>(100);
                let mut available_connections = available_connections.lock().await;

                available_connections.insert(connection_id, sender.clone());
                drop(available_connections);

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

                let _ = host_sender.send(TcpFrame::IncomingSocket { connection_id }).await;

                loop {
                    tokio::select! {
                        msg = receiver.recv() => {
                            if msg.is_none() {
                                break;
                            }

                             let _ = writer.write_buf(&mut msg.unwrap()).await;
                        },
                        frame = rx.next() => {
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
                    };
                }

                info!("BBBBBBBBBBBBBBBBBBBBBBBBBBBB");
                let _ = host_sender.send(TcpFrame::RemoteSocketDisconnected { connection_id }).await;
            });
        }

    }
}