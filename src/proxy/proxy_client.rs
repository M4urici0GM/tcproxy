use bytes::{Buf, Bytes, BytesMut};
use tracing::log::warn;
use std::collections::HashMap;
use std::io::Cursor;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Receiver, Sender, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_stream::{StreamExt, StreamMap};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, trace};
use uuid::Uuid;

use crate::codec::TcpFrame;
use crate::server::ProxyClientState;
use crate::tcp::ListenerUtils;
use crate::{PortManager, Result};

#[derive(Debug)]
pub struct ProxyClient {
    pub(crate) listen_ip: Ipv4Addr,
    pub(crate) remote_ip: SocketAddr,
    pub(crate) state: Arc<ProxyClientState>,
    pub(crate) port_manager: Arc<PortManager>,
}

type TcpFrameStream = Pin<Box<dyn tokio_stream::Stream<Item = TcpFrame> + Send>>;

struct FrameReader<'a> {
    buffer: BytesMut,
    reader: &'a mut OwnedReadHalf,
}

struct TcpConnectionStream {
    connection_id: Uuid,
    stream: TcpFrameStream,
}

impl<'a> FrameReader<'a> {
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

    pub async fn receive_frame(&mut self) -> Result<Option<TcpFrame>> {
        loop {
            if let Some(frame) = self.parse_frame().await? {
                return Ok(Some(frame));
            }

            if 0 == self.reader.read_buf(&mut self.buffer).await? {
                trace!("read 0 bytes from client.");
                if self.buffer.is_empty() {
                    debug!("received 0 bytes from client, and buffer is empty.");
                    return Ok(None);
                }

                return Err("connection reset by peer.".into());
            }
        }
    }
}

impl ProxyClient {
    pub fn new(
        listen_ip: Ipv4Addr,
        remote_ip: SocketAddr,
        port_manager: PortManager,
        state: Arc<ProxyClientState>,
    ) -> Self {
        Self {
            listen_ip,
            remote_ip,
            state,
            port_manager: Arc::new(port_manager),
        }
    }

    fn create_frame_reader(
        mut connection_reader: OwnedReadHalf,
        target_ip: Ipv4Addr,
        client_sender: Sender<TcpFrame>,
        port_manager: Arc<PortManager>,
        proxy_state: Arc<ProxyClientState>,
        cancellation_token: CancellationToken,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut frame_reader = FrameReader {
                reader: &mut connection_reader,
                buffer: BytesMut::with_capacity(1024 * 8),
            };

            while !cancellation_token.is_cancelled() {
                let maybe_frame = match frame_reader.receive_frame().await {
                    Ok(frame) => frame,
                    Err(err) => {
                        debug!("error when reading frame from client. {}", err);
                        break;
                    }
                };

                let frame = match maybe_frame {
                    Some(f) => f,
                    None => {
                        break;
                    }
                };

                match frame {
                    TcpFrame::Ping => {
                        debug!("sending pong to client.");
                        let client_sender = client_sender.clone();
                        tokio::spawn(async move {
                            let _ = client_sender.send(TcpFrame::Pong).await;
                        });
                    }
                    TcpFrame::LocalClientDisconnected { connection_id } => {
                        debug!("connection {} disconnected from client", connection_id);
                        match proxy_state.remove_connection(connection_id) {
                            Some((_, token)) => {
                                token.cancel();
                                debug!("cancelled task for connection {}", connection_id);
                            },
                            None => {
                                warn!("connection {} not found on connection state.", connection_id);
                            },
                        }

                        debug!("removed connection {} from connection state", connection_id);
                    }
                    TcpFrame::DataPacketClient {
                        connection_id,
                        buffer,
                    } => {
                        let (connection_sender, _) = match proxy_state.get_connection(connection_id) {
                            Some(sender) => sender,
                            None => {
                                continue;
                            }
                        };

                        let _ = connection_sender.send(buffer).await;
                        drop(connection_sender);
                    }
                    TcpFrame::ClientConnected => {
                        let target_port = match port_manager.get_port().await {
                            Ok(port) => port,
                            Err(_) => {
                                debug!("server cannot listen to more ports. port limit reached.");
                                let _ = client_sender.send(TcpFrame::PortLimitReached).await;
                                continue;
                            }
                        };

                        let client_token = cancellation_token.child_token();
                        let listener = ListenerUtils::new(target_ip, target_port);

                        let state = proxy_state.clone();
                        let client_sender = client_sender.clone();
                        let port_manager = port_manager.clone();

                        tokio::spawn(async move {
                            // TODO: send nack message to client if bind fails.
                            let (tcp_listener, addr) = match listener.bind().await {
                                Ok(listener) => {
                                    let addr = listener.local_addr().unwrap();
                                    (listener, addr)
                                },
                                Err(err) => {
                                    error!(
                                        "failed to listen to {}:{} {}",
                                        target_ip, target_port, err
                                    );
                                    return;
                                }
                            };

                            let proxy_listener_task = tokio::spawn(async move {
                                loop {
                                    let (connection, connection_addr) = match listener.accept(&tcp_listener).await {
                                        Ok(connection) => connection,
                                        Err(err) => {
                                            error!(
                                                "failed to accept socket. {}: {}",
                                                listener.listen_ip(),
                                                err
                                            );
                                            debug!(
                                                "closing proxy listener {}: {}",
                                                listener.listen_ip(),
                                                err
                                            );
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
    
                                    state.insert_connection(connection_id, connection_sender.clone(), CancellationToken::new());
                                    let _ = client_sender
                                        .send(TcpFrame::IncomingSocket { connection_id })
                                        .await;
    
                                    let client_sender = client_sender.clone();
                                    tokio::spawn(async move {
                                        let (notify_shutdown, _) = broadcast::channel::<()>(1);
                                        let (mut reader, mut writer) = connection.into_split();
                                        let reader_task = tokio::spawn(async move {
                                            let mut buffer = BytesMut::with_capacity(1024 * 8);
                                            loop {
                                                let bytes_read = match reader.read_buf(&mut buffer).await {
                                                    Ok(read) => read,
                                                    Err(err) => {
                                                        trace!("failed to read from connection {}: {}",connection_id, err);
                                                        break;
                                                    }
                                                };
    
                                                if 0 == bytes_read {
                                                    trace!("reached end of stream from connection {}", connection_id);
                                                    drop(reader);
                                                    break;
                                                }
    
                                                let buffer = BytesMut::from(&buffer[..bytes_read]);
                                                let frame = TcpFrame::DataPacketHost {
                                                    connection_id,
                                                    buffer,
                                                };
                                                match client_sender.send(frame).await {
                                                    Ok(_) => {}
                                                    Err(err) => {
                                                        error!(
                                                            "failed to send frame to client. {}",
                                                            err
                                                        );
                                                        break;
                                                    }
                                                }
                                            }
    
                                            trace!("received stop signal.");
                                        });
    
                                        let writer_task = tokio::spawn(async move {
                                            while let Some(mut buffer) = connection_receiver.recv().await {
                                                match writer.write_buf(&mut buffer).await {
                                                    Ok(written) => trace!(
                                                        "written {} bytes to {}",
                                                        written,
                                                        connection_addr
                                                    ),
                                                    Err(err) => {
                                                        error!(
                                                            "failed to write into {}: {}",
                                                            connection_addr, err
                                                        );
                                                        break;
                                                    }
                                                };
        
                                                let _ = writer.flush().await;
                                            }
                                        });
    
                                        tokio::select! {
                                            _ = reader_task => {},
                                            _ = writer_task => {},
                                        };
    
                                        debug!("received none from connection {}, aborting", connection_id);
                                        drop(notify_shutdown);
                                    });
                                }
                            });

                            tokio::select! {
                                _ = proxy_listener_task => {},
                                _ = client_token.cancelled() => {},
                            }

                            debug!("closing proxy server listening at {}",addr);
                            port_manager.remove_port(target_port);
                        });
                    }
                    _ => {
                        debug!("invalid frame received.");
                    }
                }
            }
        })
    }

    fn create_frame_writer(
        mut client_reader: Receiver<TcpFrame>,
        mut connection_writer: OwnedWriteHalf,
        cancellation_token: CancellationToken,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    frame = client_reader.recv() => {
                        match frame {
                            Some(frame) => {
                                let mut buffer = frame.to_buffer();
                                match connection_writer.write_buf(&mut buffer).await {
                                    Ok(written) => debug!("written {} bytes to client.", written),
                                    Err(err)=> {
                                        error!("failed to write frame to client: {}", err);
                                        break;
                                    }
                                };

                                let _ = connection_writer.flush().await;
                            },
                            None => {
                                debug!("received None from client channel");
                                break;
                            }
                        }
                    },
                    _ = cancellation_token.cancelled() => {
                        break;
                    },
                }
            }
        })
    }

    pub async fn start_streaming(
        &mut self,
        tcp_stream: TcpStream,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        let (connection_reader, connection_writer) = tcp_stream.into_split();
        let local_cancellation_token = CancellationToken::new();

        let (client_sender, client_reader) = mpsc::channel::<TcpFrame>(1000);
        let task2 = ProxyClient::create_frame_reader(
            connection_reader,
            self.listen_ip,
            client_sender.clone(),
            self.port_manager.clone(),
            self.state.clone(),
            local_cancellation_token.child_token(),
        );

        let task3 = ProxyClient::create_frame_writer(
            client_reader,
            connection_writer,
            local_cancellation_token.child_token(),
        );

        tokio::select! {
            res = task2 => debug!("ProxyClient::create_frame_reader task completed with {:?}", res),
            res = task3 => debug!("ProxyClient::create_frame_writer task completed with {:?}", res),
            _ = cancellation_token.cancelled() => debug!("received global stop signal.."),
        };

        local_cancellation_token.cancel();
        Ok(())
    }
}
