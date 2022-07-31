use bytes::{BufMut, BytesMut};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tokio_stream::StreamMap;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tcproxy::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{self, Duration, Instant};
use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::sync::Mutex;
use tokio_stream::Stream;
use tracing::{debug, error, info};
use uuid::Uuid;

use tcproxy::codec::{TcpFrame, TcpFrameCodec};

async fn send_connected_frame(writer: &mut SplitSink<Framed<TcpStream, TcpFrameCodec>, TcpFrame>) {
    match writer.send(TcpFrame::ClientConnected).await {
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
                },
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
    let tcp_connection = match TcpStream::connect("144.217.14.8:8080").await {
        Ok(stream) => {
            debug!("Connected to server..");
            stream
        },
        Err(err) => {
            error!("Failed to connect to server. Check you network connection and try again.");
            return Err(format!("Failed when connecting to server: {}", err).into());
        }
    };

    let cancellation_token = CancellationToken::new();
    let mut connections: HashMap<Uuid, (Sender<BytesMut>, CancellationToken)> = HashMap::new();
    let transport = Framed::new(tcp_connection, TcpFrameCodec { source: "CLIENT".to_string() });
    let (mut transport_writer, mut transport_reader) = transport.split();
    send_connected_frame(&mut transport_writer).await;

    let (main_sender, mut main_receiver) = mpsc::channel::<TcpFrame>(100);

    let token_clone = cancellation_token.child_token();
    tokio::spawn(async move {
        while let Some(msg) = main_receiver.recv().await {
            let _ = transport_writer.send(msg).await;
        }

        debug!("Reached end of stream.");
    });


    start_ping(main_sender.clone());

    let mut stream_map = StreamMap::new();
    while !token_clone.is_cancelled() {
        tokio::select! {
            Some((_, msg)) = stream_map.next() => {
                debug!("AAAAAAAAAAAAA");
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
            msg = transport_reader.next() => {
                debug!("received new frame from server.");
                let maybe_msg = match msg {
                    Some(msg) => msg,
                    None => {
                        break;
                    }
                };

                let msg_result = match maybe_msg {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("error when receiving new message. {}", err);
                        info!("Connection with server was shut down. closing..");
                        cancellation_token.cancel();
                        main_sender.closed().await;
                        break;
                    }
                };

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

                        let connection = match TcpStream::connect("127.0.0.1:80").await {
                            Ok(stream) => stream,
                            Err(err) => {
                                debug!("Error when connecting to {}: {}. Aborting connection..", "127.0.0.1:80", err);
                                let _ = main_sender
                                    .send(TcpFrame::ClientUnableToConnect { connection_id })
                                    .await;
                                return Ok(());
                            }
                        };
                    
                        let (mut stream_reader, mut stream_writer) = connection.into_split();
                    
                        let token = cancellation_token.child_token();
                        let socket_stream = Box::pin(async_stream::stream! {
                            let mut buffer = BytesMut::with_capacity(1024 * 8);
                            while !token.is_cancelled() {
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
                    
                                debug!("read {} bytes from stream", bytes_read);
                                if 0 == bytes_read {
                                    debug!("reached end of stream");
                                    break;
                                }
                    
                                buffer.truncate(bytes_read);
                                yield TcpFrame::DataPacketClient {
                                    connection_id: connection_id,
                                    buffer: buffer.clone(),
                                };
                            }
                        });
                
                        stream_map.insert(connection_id, socket_stream);
                        let sender = main_sender.clone();
                        tokio::spawn(async move {
                            while !cancellation_token.is_cancelled() {
                                let result = reader.recv().await;
                                if result == None {
                                    debug!("test");
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
