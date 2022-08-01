use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use std::pin::Pin;
use std::sync::{Arc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::{Sender, Receiver, self};
use tokio::task::JoinHandle;
use tokio_stream::{StreamMap, StreamExt};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use std::io::Cursor;
use bytes::{BytesMut, Buf};
use tracing::{trace, debug, error};

use crate::server::ProxyClientState;
use crate::tcp::ListenerUtils;
use crate::{PortManager, Result};
use crate::codec::TcpFrame;

#[derive(Debug)]
pub struct ProxyClient {
    pub(crate) listen_ip: Ipv4Addr,
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

}


impl ProxyClient {
    pub fn new(listen_ip: Ipv4Addr, remote_ip: SocketAddr, port_manager: PortManager, state: Arc<ProxyClientState>) -> Self {
        Self {
            listen_ip,
            port_manager: Arc::new(port_manager),
        }
    }

    fn start_reading(
        mut reader: OwnedReadHalf,
        channel_writer: Sender<BytesMut>,
        listen_ip: Ipv4Addr,
        port_manager: Arc<PortManager>,
        new_connections_writer: Sender<TcpConnectionStream>,
        disconnected_connections_writer: Sender<Uuid>,
        connections: Arc<Mutex<HashMap<Uuid, (Sender<BytesMut>, CancellationToken)>>>
    ) -> JoinHandle<()> {

        tokio::spawn(async move {
            let mut frame_reader = FrameReader {
                reader: &mut reader,
                buffer: BytesMut::with_capacity(1024 * 8),
            };

            loop {
                let maybe_frame = frame_reader.receive_frame().await.unwrap();
                let frame = match maybe_frame {
                    Some(f) => f,
                    None => {
                        break;
                    }
                };
                
                debug!("received new frame from client {}", frame);
                match frame {
                    TcpFrame::ClientConnected => {
                        let port = port_manager.get_port().await.unwrap();
                        
                        let stream_map = new_connections_writer.clone();
                        let connections = connections.clone();
    
                        let listener = ListenerUtils {
                            ip: listen_ip,
                            port,
                        };

                        let disconnected_connections_writer = disconnected_connections_writer.clone();
                        tokio::spawn(async move {
                            let tcp_listener = listener.bind().await.unwrap();
                            loop {
                                let token = CancellationToken::new();

                                let cancellation_token = token.child_token();
                                let (connection, _) = listener.accept(&tcp_listener).await.unwrap();
                                let (reader, mut writer) = connection.into_split();
                                let connection_id = Uuid::new_v4();
    
                                let (channel_writer, mut channel_reader) = mpsc::channel::<BytesMut>(100);
                                let stream = create_socket_stream(connection_id, reader);
    
                                let _ = stream_map.send(TcpConnectionStream { connection_id, stream }).await;
                                connections.lock().await.insert(connection_id, (channel_writer, token));
    
                                let disconnected_connections_writer = disconnected_connections_writer.clone();
                                tokio::spawn(async move {
                                    while !cancellation_token.is_cancelled() {
                                        if let Some(mut buff) = channel_reader.recv().await {
                                            let _ = writer.write_buf(&mut buff).await;
                                        }
                                    }

                                    debug!("remote client disconnected");
                                    let _ = disconnected_connections_writer.send(connection_id).await;
                                });
                            }
                        });

                        let buffer = TcpFrame::ClientConnectedAck { port }.to_buffer();
                        let _ = channel_writer.send(buffer).await;
                    },
                    TcpFrame::LocalClientDisconnected { connection_id } => {
                        let wrapper = connections.lock().await;
                        if !wrapper.contains_key(&connection_id) {
                            return;
                        }

                        let (sender, cancellation_token) = wrapper.get(&connection_id).unwrap();
                        let _ = sender.closed().await;
                        cancellation_token.cancel();
                    },
                    TcpFrame::DataPacketClient { connection_id, buffer } => {
                        debug!("received new frame from client.");
                        let wrapper = connections.lock().await;
                        if !wrapper.contains_key(&connection_id) {
                            debug!("connection {} not found.", connection_id);
                            return;
                        }

                        let (sender, _) = wrapper.get(&connection_id).unwrap();
                        let _ = sender.send(buffer).await;
                    },
                    _ => {},
                }
            }
        })
    }

    fn start_writing(mut channel_reader: Receiver<BytesMut>, mut connection_writer: OwnedWriteHalf) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(mut msg) = channel_reader.recv().await {
                match connection_writer.write_buf(&mut msg).await {
                    Ok(s) => {
                        trace!("written {} bytes to client.", s);
                    },
                    Err(err) => {
                        trace!("error when writing to client. {}", err);
                        break;
                    }
                };
            }
        })
    }
    
    pub async fn start_streaming(&mut self, tcp_stream: TcpStream, cancellation_token: CancellationToken) -> Result<()> {
        let (connection_reader, connection_writer) = tcp_stream.into_split();

        let (channel_writer, channel_reader) = mpsc::channel::<BytesMut>(100);
        let (new_connection_writer, mut new_connection_reader) = mpsc::channel::<TcpConnectionStream>(100);
        let (disconnected_connections_writer, mut disconnected_connections_reader) = mpsc::channel::<Uuid>(100);
        let mut stream_map: StreamMap<Uuid, TcpFrameStream> = StreamMap::new();
        let connections: Arc<Mutex<HashMap<Uuid, (Sender<BytesMut>, CancellationToken)>>> = Arc::new(Mutex::new(HashMap::new()));

        let task1 = ProxyClient::start_reading(
            connection_reader,
            channel_writer.clone(),
            self.listen_ip.clone(),
            self.port_manager.clone(),
            new_connection_writer.clone(),
            disconnected_connections_writer.clone(),
            connections.clone());


        let task2 = ProxyClient::start_writing(channel_reader, connection_writer);
        let task3 = tokio::spawn(async move {
            loop {
                tokio::select!{
                    Some(connection_id) = disconnected_connections_reader.recv() => {
                        stream_map.remove(&connection_id);
                    },
                    Some(msg) = new_connection_reader.recv() => {
                      stream_map.insert(msg.connection_id, msg.stream);
                    },
                    Some((connection_id, frame)) = stream_map.next() => {
                        debug!("received new frame from connection {}", connection_id);

                        let buff = frame.to_buffer();
                        let _ = channel_writer.send(buff).await;
                    },
                    _ = cancellation_token.cancelled() => {
                        break;
                    },
                }
            }
        });

        tokio::select! {
            _ = task1 => {},
            _ = task2 => {},
            _ = task3 => {},
        };

        Ok(())
    }
}

fn create_socket_stream(connection_id: Uuid, mut reader: OwnedReadHalf) -> TcpFrameStream {
    Box::pin(async_stream::stream! {
        loop {
            let mut buffer = BytesMut::with_capacity(1024 * 8);
    
            let bytes_read = match reader.read_buf(&mut buffer).await {
                Ok(size) => size,
                Err(err) => {
                    error!(
                        "Failed when reading from connection {}: {}",
                        connection_id,
                        err
                    );
                    break;
                }
            };

            debug!("read {} bytes from target socket", bytes_read);
            if 0 == bytes_read {
                debug!("reached end of stream");
                break;
            }

            buffer.truncate(bytes_read);
            yield TcpFrame::DataPacketHost {
                connection_id: connection_id,
                buffer: buffer,
            };

        }
    })
}