use bytes::{Buf, BufMut, BytesMut};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::io::Cursor;
use tcproxy::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinHandle;
use tokio::time::{self, Duration, Instant};
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace};
use uuid::Uuid;

use tcproxy::codec::{FrameError, TcpFrame};

async fn send_connected_frame(writer: &mut OwnedWriteHalf) {
    match writer
        .write_buf(&mut TcpFrame::ClientConnected.to_buffer())
        .await
    {
        Ok(_) => {
            info!("send initial frame to server..");
        }
        Err(err) => {
            error!("Failed sending initial frame to server: {}", err);
        }
    };
}

struct LocalConnection {
    connection_id: Uuid,
    sender: Sender<TcpFrame>,
}

struct FrameReader<'a> {
    buffer: BytesMut,
    reader: &'a mut OwnedReadHalf,
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
            Err(FrameError::Incomplete) => Ok(None),
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

impl LocalConnection {
    pub async fn read_from_local_connection(
        &mut self,
        mut reader: Receiver<BytesMut>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        let connection = match TcpStream::connect("127.0.0.1:3337").await {
            Ok(stream) => stream,
            Err(err) => {
                debug!(
                    "Error when connecting to {}: {}. Aborting connection..",
                    "127.0.0.1:80", err
                );
                let _ = self
                    .sender
                    .send(TcpFrame::ClientUnableToConnect {
                        connection_id: self.connection_id,
                    })
                    .await;

                return Ok(());
            }
        };

        let (mut stream_reader, mut stream_writer) = connection.into_split();

        let thread_sender = self.sender.clone();
        let connection_id = self.connection_id.clone();
        let token_clone = cancellation_token.child_token();
        let task1 = tokio::spawn(async move {
            let mut buffer = BytesMut::with_capacity(1024 * 8);
            while !token_clone.is_cancelled() {
                let bytes_read = match stream_reader.read_buf(&mut buffer).await {
                    Ok(size) => size,
                    Err(err) => {
                        error!(
                            "Failed when reading from connection {}: {}",
                            connection_id, err
                        );
                        break;
                    }
                };

                if 0 == bytes_read {
                    debug!("reached end of stream");
                    break;
                }

                buffer.truncate(bytes_read);
                let tcp_frame = TcpFrame::DataPacketClient {
                    connection_id,
                    buffer: buffer.clone(),
                };

                match thread_sender.send(tcp_frame).await {
                    Ok(_) => {}
                    Err(err) => {
                        debug!(
                            "failed when sending frame to main thread loop. connection {}: {}",
                            connection_id, err
                        );
                        return;
                    }
                };
            }
        });

        let connection_id = self.connection_id.clone();
        let token_clone = cancellation_token.child_token();
        let task2 = tokio::spawn(async move {
            while !token_clone.is_cancelled() {
                let result = reader.recv().await;
                if result == None {
                    break;
                }

                let mut msg = result.unwrap();
                let bytes_written = match stream_writer.write_buf(&mut msg).await {
                    Ok(a) => a,
                    Err(err) => {
                        debug!("Error when writing buffer to {}: {}", connection_id, err);
                        break;
                    }
                };

                let _ = stream_writer.flush().await;
                debug!("written {} bytes to target stream", bytes_written);
            }
        });

        tokio::select! {
            _ = task1 => {},
            _ = task2 => {},
        };

        if cancellation_token.is_cancelled() {
            return Ok(());
        }

        debug!("connection {} disconnected!", self.connection_id);
        let _ = self
            .sender
            .send(TcpFrame::LocalClientDisconnected { connection_id: self.connection_id })
            .await;

        Ok(())
    }
}

fn start_ping(sender: Sender<TcpFrame>) {
    tokio::spawn(async move {
        loop {
            info!("Waiting for next ping to occur");
            time::sleep_until(Instant::now() + Duration::from_secs(10)).await;
            match sender.send(TcpFrame::Ping).await {
                Ok(_) => {
                    info!("Sent ping frame..");
                }
                Err(err) => {
                    error!("Failed to send ping. aborting. {}", err);
                    break;
                }
            };
        }
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let tcp_connection = match TcpStream::connect("192.168.0.221:8080").await {
        Ok(stream) => {
            debug!("Connected to server..");
            stream
        }
        Err(err) => {
            error!("Failed to connect to server. Check you network connection and try again.");
            return Err(format!("Failed when connecting to server: {}", err).into());
        }
    };

    let mut connections: HashMap<Uuid, (Sender<BytesMut>, CancellationToken)> = HashMap::new();
    let (mut reader, mut writer) = tcp_connection.into_split();
    send_connected_frame(&mut writer).await;

    let (main_sender, mut main_receiver) = mpsc::channel::<TcpFrame>(100);

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = main_receiver.recv().await {
            let _ = writer.write_buf(&mut msg.to_buffer()).await;
        }

        info!("Reached end of stream.");
    });

    start_ping(main_sender.clone());
    let foward_task = tokio::spawn(async move {
        let mut frame_reader = FrameReader {
            buffer: BytesMut::with_capacity(1024 * 8),
            reader: &mut reader,
        };

        loop {
            let maybe_frame = match frame_reader.receive_frame().await {
                Ok(f) => f,
                Err(err) => {
                    debug!("error when reading frame.. {}", err);
                    break;
                }
            };

            let msg = match maybe_frame {
                Some(f) => f,
                None => {
                    debug!("received none from framereader.");
                    break;
                }
            };

            debug!("received new frame from server: {}", msg);
            match msg {
                TcpFrame::ClientConnectedAck { port } => {
                    info!("Remote proxy listening in {}:{}", "127.0.0.1", port);
                }
                TcpFrame::DataPacketHost {
                    connection_id,
                    buffer,
                } => {
                    debug!("received new packet from {}", connection_id);
                    if !connections.contains_key(&connection_id) {
                        debug!("connection {} not found!", connection_id);
                        continue;
                    }

                    let (sender, _) = connections.get(&connection_id).unwrap();
                    let _ = sender.send(buffer).await;
                }
                TcpFrame::IncomingSocket { connection_id } => {
                    debug!("new connection received!");
                    let token = CancellationToken::new();
                    let cancellation_token = token.child_token();
                    let (sender, reader) = mpsc::channel::<BytesMut>(100);

                    connections.insert(connection_id, (sender, token));

                    let mut local_connection = LocalConnection {
                        connection_id,
                        sender: main_sender.clone(),
                    };
                    let cancellation_token = cancellation_token.child_token();
                    tokio::spawn(async move {
                        let _ = local_connection
                            .read_from_local_connection(reader, cancellation_token.child_token())
                            .await;
                    });
                }
                TcpFrame::RemoteSocketDisconnected { connection_id } => {
                    let (_, cancellation_token) = match connections.remove(&connection_id) {
                        Some(item) => item,
                        None => {
                            debug!("connection not found {}", connection_id);
                            continue;
                        }
                    };

                    cancellation_token.cancel();
                }
                packet => {
                    debug!("invalid data packet received. {}", packet);
                }
            }
        }
    });

    tokio::select! {
        _ = receive_task => {
            debug!("Receive task finished.");
        },
        res = foward_task => {
            debug!("Forward to server task finished. {:?}", res);
        },
    };

    Ok(())
}
