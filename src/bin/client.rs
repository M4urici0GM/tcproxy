use bytes::{BufMut, BytesMut};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use tcproxy::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::sync::Mutex;
use tokio_util::codec::Framed;
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

struct LocalConnection {
    connection_id: Uuid,
    sender: Sender<TcpFrame>,
}

impl LocalConnection {
    pub async fn read_from_local_connection(&mut self, reader: &mut Receiver<BytesMut>) -> Result<()> {
        let mut connection = match TcpStream::connect("127.0.0.1:80").await {
            Ok(stream) => stream,
            Err(err) => {
                debug!("Error when connecting to {}: {}. Aborting connection..", "127.0.0.1:80", err);
                let _ = self.sender
                    .send(TcpFrame::ClientUnableToConnect { connection_id: self.connection_id })
                    .await;
                return Ok(());
            }
        };

        let (mut stream_reader, mut stream_writer) = connection.split();

        let thread_sender = self.sender.clone();
        let connection_id = self.connection_id.clone();
        let task1 = async move {
            let mut buffer = BytesMut::with_capacity(1024 * 8);
            loop {
                let bytes_read = match stream_reader
                    .read_buf(&mut buffer)
                    .await
                {
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

                if 0 == bytes_read {
                    debug!("reached end of stream");
                    break;
                }

                buffer.truncate(bytes_read);
                let tcp_frame = TcpFrame::DataPacket {
                    connection_id,
                    buffer: buffer.clone(),
                };

                match thread_sender.send(tcp_frame).await {
                    Ok(_) => {}
                    Err(err) => {
                        debug!("failed when sending frame to main thread loop. connection {}: {}", connection_id, err);
                        return;
                    }
                };
            }
        };

        let connection_id = self.connection_id.clone();
        let task2 = async move {
            while let Some(mut msg) = reader.recv().await {
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
        };

        tokio::select! {
            _ = task1 => {},
            _ = task2 => {},
        };

        debug!("connection {} disconnected!", self.connection_id);
        let _ = self.sender
            .send(TcpFrame::LocalClientDisconnected { connection_id: self.connection_id })
            .await;

        Ok(())
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let tcp_connection = match TcpStream::connect("127.0.0.1:8080").await {
        Ok(stream) => stream,
        Err(err) => {
            return Err(format!("Failed when connecting to server: {}", err).into());
        }
    };

    let connections: Arc<Mutex<HashMap<Uuid, Sender<BytesMut>>>> = Arc::new(Mutex::new(HashMap::new()));
    let transport = Framed::new(tcp_connection, TcpFrameCodec);
    let (mut transport_writer, mut transport_reader) = transport.split();
    send_connected_frame(&mut transport_writer).await;

    let (main_sender, mut main_receiver) = mpsc::channel::<TcpFrame>(100);

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = main_receiver.recv().await {
            let _ = transport_writer.send(msg).await;
        }
    });

    let foward_task = tokio::spawn(async move {
        loop {
            while let Some(msg) = transport_reader.next().await {
                if let Ok(..) = msg {
                    let msg = msg.unwrap();
                    match msg {
                        TcpFrame::ClientConnectedAck { port } => {
                            debug!("Remote proxy listening in {}:{}", "127.0.0.1", port);
                        }
                        TcpFrame::DataPacket {
                            connection_id,
                            buffer,
                        } => {
                            info!("received new packet from {}", connection_id);
                            let mutex_guard = connections.lock().await;
                            if !mutex_guard.contains_key(&connection_id) {
                                debug!("connection {} not found!", connection_id);
                                continue;
                            }

                            let sender = mutex_guard.get(&connection_id).unwrap();
                            let _ = sender.send(buffer).await;
                        }
                        TcpFrame::IncomingSocket { connection_id } => {
                            debug!("new connection received!");
                            let (sender, mut reader) = mpsc::channel::<BytesMut>(100);

                            let mut mutex_lock = connections.lock().await;
                            mutex_lock.insert(connection_id, sender);

                            let mut local_connection = LocalConnection { connection_id, sender: main_sender.clone() };
                            tokio::spawn(async move {
                                let _ = local_connection.read_from_local_connection(&mut reader).await;                                
                            });
                        },
                        packet => {
                            debug!("invalid data packet received. {}", packet);
                        }
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = receive_task => {},
        _ = foward_task => {},
    };


    Ok(())
}
