use bytes::{Buf, BytesMut};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::io::Cursor;
use std::pin::Pin;
use tcproxy::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};
use tokio::time::{self, Duration, Instant};
use tokio_stream::Stream;
use tokio_stream::StreamMap;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use uuid::Uuid;

use tcproxy::codec::{FrameError, TcpFrame};

async fn send_connected_frame(writer: &mut OwnedWriteHalf) {
    let mut buff = TcpFrame::ClientConnected.to_buffer();
    match writer.write_buf(&mut buff).await {
        Ok(_) => {
            info!("send initial frame to server..");
        }
        Err(err) => {
            error!("Failed sending initial frame to server: {}", err);
        }
    };
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
    let tcp_connection = match TcpStream::connect("127.0.0.1:8080").await {
        Ok(stream) => {
            debug!("Connected to server..");
            stream
        }
        Err(err) => {
            error!("Failed to connect to server. Check you network connection and try again.");
            return Err(format!("Failed when connecting to server: {}", err).into());
        }
    };

    let cancellation_token = CancellationToken::new();
    let mut connections: HashMap<Uuid, (Sender<BytesMut>, CancellationToken)> = HashMap::new();
    let (mut reader, mut writer) = tcp_connection.into_split();

    send_connected_frame(&mut writer).await;

    let (main_sender, mut main_receiver) = mpsc::channel::<TcpFrame>(100);

    let token_clone = cancellation_token.child_token();
    tokio::spawn(async move {
        while let Some(msg) = main_receiver.recv().await {
            let mut buff = msg.to_buffer();
            let bytes_written = match writer.write_buf(&mut buff).await {
                Ok(s) => s,
                Err(err) => {
                    error!("failed when writing data to server.. {}", err);
                    break;
                }
            };

            debug!("written {} bytes to server..", bytes_written);
        }

        debug!("Reached end of stream.");
    });

    let mut socket_stream = Box::pin(async_stream::stream! {
        loop {
            let mut buffer = BytesMut::with_capacity(1024 * 8);
            let bytes_read = match reader.read_buf(&mut buffer).await {
                Ok(r) => r,
                Err(err) => {
                    debug!("Failed to read buffer from socket.");
                    break;
                }
            };

            if 0 == bytes_read {
                if buffer.is_empty() {
                    continue;
                }

                debug!("connection reset by peer");
                break;
            }

            let mut cursor = Cursor::new(&buffer[..]);
            let result: Result<Option<TcpFrame>> = match TcpFrame::check(&mut cursor) {
                Ok(_) => {
                    let position = cursor.position() as usize;
                    cursor.set_position(0);

                    let frame = match TcpFrame::parse(&mut cursor) {
                        Ok(f) => f,
                        Err(err) => {
                            debug!("failed to parse frame: {}", err);
                            break;
                        }
                    };

                    buffer.advance(position);
                    Ok(Some(frame))
                }
                Err(FrameError::Incomplete) => Ok(None),
                Err(err) => Err(err.into()),
            };

            let maybe_frame = match result {
                Ok(Some(f)) => f,
                Ok(None) => {
                    continue;
                },
                Err(err) => {
                    break;
                }
            };

            yield maybe_frame;
        }
    });

    start_ping(main_sender.clone());

    let mut stream_map = StreamMap::new();
    while !token_clone.is_cancelled() {
        tokio::select! {
            Some((_, msg)) = stream_map.next() => {
                match  main_sender.send(msg).await {
                    Ok(_) => {
                        debug!("send frame to server.. ");
                    },
                    Err(err) => {
                        error!("failed to send frame to server. {}", err);
                        cancellation_token.cancel();
                        main_sender.closed().await;
                    }
                };
            },
            msg = socket_stream.next() => {
                let msg_result = match msg {
                    Some(msg) => msg,
                    None => {
                        info!("Connection with server was shut down. closing..");
                        cancellation_token.cancel();
                        main_sender.closed().await;
                        break;
                    }
                };

                debug!("received new frame from server. {}", msg_result);
                match msg_result {
                    TcpFrame::ClientConnectedAck { port } => {
                        debug!("Remote proxy listening in {}:{}", "127.0.0.1", port);
                    }
                    TcpFrame::DataPacketHost {
                        connection_id,
                        buffer,
                    } => {
                        info!("received new packet from {} CLIENT", connection_id);
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
                        let (sender, mut reader) = mpsc::channel::<BytesMut>(100);

                        connections.insert(connection_id, (sender, token));

                        let sender = main_sender.clone();
                        tokio::spawn(async move {
                            let connection = match TcpStream::connect("127.0.0.1:80").await {
                                Ok(stream) => {
                                    debug!("connected to target socket.");
                                    stream
                                },
                                Err(err) => {
                                    debug!("Error when connecting to {}: {}. Aborting connection..", "127.0.0.1:80", err);
                                    let _ = sender
                                        .send(TcpFrame::ClientUnableToConnect { connection_id })
                                        .await;
                                    return;
                                }
                            };
    
                            let (mut stream_reader, mut stream_writer) = connection.into_split();
    
                            let token = cancellation_token.child_token();
                            let socket_stream = Box::pin(async_stream::stream! {
                                while !token.is_cancelled() {
                                let mut buffer = BytesMut::with_capacity(1024 * 8);
    
                                    let bytes_read = match stream_reader.read_buf(&mut buffer).await {
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
                                    yield TcpFrame::DataPacketClient {
                                        connection_id: connection_id,
                                        buffer: buffer,
                                    };
                                }
                            });
    
                            stream_map.insert(connection_id, socket_stream);

                            while !cancellation_token.is_cancelled() {
                                let result = reader.recv().await;
                                if result == None {
                                    break;
                                }

                                let mut msg = result.unwrap();
                                let bytes_written =
                                    match stream_writer.write_buf(&mut msg).await {
                                        Ok(a) => a,
                                        Err(err) => {
                                            debug!("Error when writing buffer to {}: {}", connection_id, err);
                                            break;
                                        }
                                    };

                                debug!("written {} bytes to target stream", bytes_written);
                            }

                            debug!("Received cancel signal.. aborting");
                            debug!("connection {} disconnected!", connection_id);
                            let _ = sender
                                .send(TcpFrame::LocalClientDisconnected { connection_id: connection_id })
                                .await;
                        });
                    },
                    TcpFrame::RemoteSocketDisconnected { connection_id } => {
                        let (_, cancellation_token) = match connections.remove(&connection_id) {
                            Some(item) => item,
                            None => {
                                debug!("connection not found {}", connection_id);
                                continue;
                            }
                        };

                        cancellation_token.cancel();
                    },
                    TcpFrame::Pong => {
                        debug!("received pong.");
                    },
                    packet => {
                        debug!("invalid data packet received. {}", packet);
                    }
                };
            }
        }
    }

    debug!("KKKKKKKKKKKKKKKKKKKKKKK");
    Ok(())
}
