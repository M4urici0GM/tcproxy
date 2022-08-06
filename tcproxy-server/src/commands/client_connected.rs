use async_trait::async_trait;
use bytes::BytesMut;
use std::{net::Ipv4Addr, sync::Arc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, trace};
use uuid::Uuid;
use tcproxy_core::{Result, TcpFrame, Command};

use crate::ProxyState;
use crate::tcp::ListenerUtils;

pub struct ClientConnectedCommand {
    pub(crate) target_ip: Ipv4Addr,
    pub(crate) sender: Sender<TcpFrame>,
    pub(crate) state: Arc<ProxyState>,
    pub(crate) cancellation_token: CancellationToken,
}

#[async_trait]
impl Command for ClientConnectedCommand {
    async fn handle(&mut self) -> Result<()> {
        let target_port = match self.state.ports.get_port().await {
            Ok(port) => port,
            Err(err) => {
                debug!("server cannot listen to more ports. port limit reached.");
                self.sender.send(TcpFrame::PortLimitReached).await?;
                return Err(err);
            }
        };

        let listener = ListenerUtils::new(self.target_ip, target_port);
        let target_ip = self.target_ip.clone();
        let state = self.state.clone();
        let client_sender = self.sender.clone();
        let cancellation_token = self.cancellation_token.child_token();

        tokio::spawn(async move {
            // TODO: send nack message to client if bind fails.
            let (tcp_listener, addr) = match listener.bind().await {
                Ok(listener) => {
                    let addr = listener.local_addr().unwrap();
                    (listener, addr)
                }
                Err(err) => {
                    error!("failed to listen to {}:{} {}", target_ip, target_port, err);
                    return;
                }
            };

            let _ = client_sender.send(TcpFrame::ClientConnectedAck { port: target_port }).await;

            let aaa = state.clone();
            let proxy_listener_task = tokio::spawn(async move {
                let semaphore = Arc::new(Semaphore::new(120));
                loop {
                    let permit = semaphore.clone().acquire_owned().await.unwrap();
                    let (connection, connection_addr) = match listener.accept(&tcp_listener).await {
                        Ok(connection) => connection,
                        Err(err) => {
                            error!("failed to accept socket. {}: {}", listener.listen_ip(), err);
                            debug!("closing proxy listener {}: {}", listener.listen_ip(), err);
                            break;
                        }
                    };

                    debug!(
                        "received new connection on proxy {} from {}",
                        listener.listen_ip(),
                        connection_addr
                    );

                    let connection_id = Uuid::new_v4();
                    let (connection_sender, mut connection_receiver) = mpsc::channel::<BytesMut>(100);

                    aaa.connections.insert_connection(
                        connection_id,
                        connection_sender,
                        CancellationToken::new(),
                    );
                    let _ = client_sender
                        .send(TcpFrame::IncomingSocket { connection_id })
                        .await;

                    let client_sender = client_sender.clone();
                    tokio::spawn(async move {
                        let (mut reader, mut writer) = connection.into_split();
                        let aa = client_sender.clone();
                        let reader_task = tokio::spawn(async move {
                            loop {
                                let mut buffer = BytesMut::with_capacity(1024 * 8);
                                let bytes_read = match reader.read_buf(&mut buffer).await {
                                    Ok(read) => read,
                                    Err(err) => {
                                        trace!(
                                            "failed to read from connection {}: {}",
                                            connection_id,
                                            err
                                        );
                                        break;
                                    }
                                };

                                if 0 == bytes_read {
                                    trace!(
                                        "reached end of stream from connection {}",
                                        connection_id
                                    );
                                    drop(reader);
                                    break;
                                }

                                buffer.truncate(bytes_read);
                                let buffer = BytesMut::from(&buffer[..]);
                                let frame = TcpFrame::DataPacketHost {
                                    connection_id,
                                    buffer,
                                    buffer_size: bytes_read as u32,
                                };
                                match aa.send(frame).await {
                                    Ok(_) => {}
                                    Err(err) => {
                                        error!("failed to send frame to client. {}", err);
                                        break;
                                    }
                                }
                            }

                            trace!("received stop signal.");
                        });

                        let writer_task = tokio::spawn(async move {
                            while let Some(mut buffer) = connection_receiver.recv().await {
                                let mut buffer = buffer.split();
                                match writer.write_buf(&mut buffer).await {
                                    Ok(written) => {
                                        trace!("written {} bytes to {}", written, connection_addr)
                                    }
                                    Err(err) => {
                                        error!("failed to write into {}: {}", connection_addr, err);
                                        break;
                                    }
                                };

                                let _ = writer.flush().await;
                            }

                            connection_receiver.close();
                        });

                        tokio::select! {
                            _ = reader_task => {},
                            _ = writer_task => {},
                        };

                        drop(permit);
                        debug!("received none from connection {}, aborting", connection_id);
                        let _ = client_sender
                            .send(TcpFrame::RemoteSocketDisconnected { connection_id })
                            .await;
                    });
                }
            });

            tokio::select! {
                _ = proxy_listener_task => {},
                _ = cancellation_token.cancelled() => {},
            }

            debug!("closing proxy server listening at {}", addr);
            state.ports.remove_port(target_port);
        });

        Ok(())
    }
}
