use std::collections::HashMap;
use std::fs::read;
use std::sync::mpsc::{channel, Sender};
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tracing::{debug, error, info};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tracing::field::debug;
use uuid::Uuid;

type Error = Box<dyn std::error::Error + Sync + Send>;
type Result<T> = std::result::Result<T, Error>;

enum Message {
    Connected,
    DataBytes(BytesMut),
    Disconnected,
}

struct DuplexTcpStream<'a> {
    left: &'a mut TcpStream,
    right: &'a mut TcpStream,
    buffer_size: usize,
}

impl<'a> DuplexTcpStream<'a> {
    pub fn join(left_stream: &'a mut TcpStream, right_stream: &'a mut TcpStream, buffer_size: Option<usize>) -> Self {
        let buffer_size = match buffer_size {
            Some(size) => size,
            None => 1024 * 8,
        };

        Self {
            left: left_stream,
            right: right_stream,
            buffer_size,
        }
    }

    pub async fn start(&'a mut self) -> Result<()> {
        let (mut source_reader, mut source_writer) = self.left.split();
        let (mut target_reader, mut target_writer) = self.right.split();

        let source_to_target = Self::start_streaming(self.buffer_size.clone(), &mut source_reader, &mut target_writer);
        let target_to_source = Self::start_streaming(self.buffer_size.clone(), &mut target_reader, &mut source_writer);

        return tokio::select! {
            stt_result = source_to_target => stt_result,
            tts_result = target_to_source => tts_result,
        };
    }

    /// dsadsa
    /// sdadsa
    async fn start_streaming(buffer_size: usize, reader: &mut ReadHalf<'a>, writer: &mut WriteHalf<'a>) -> Result<()> {
        let mut buffer = BytesMut::with_capacity(buffer_size);

        loop {
            let bytes_read = match reader.read_buf(&mut buffer).await {
                Ok(size) => size,
                Err(err) => {
                    error!("Error when reading from stream. {}", err);
                    return Err(err.into());
                }
            };


            if 0 == bytes_read {
                return Ok(());
            }

            debug!("Read {} bytes from stream", bytes_read);
            buffer.truncate(bytes_read);
            let bytes_written = match writer.write_buf(&mut buffer).await {
                Ok(size) => size,
                Err(err) => {
                    error!("Error when writing to stream..");
                    return Err(err.into());
                }
            };

            let _ = writer.flush().await;

            debug!("Written {} bytes into stream", bytes_written);
            if bytes_read != bytes_written {
                return Err("Buffer mismatch.. Closing..".into());
            }
        }
    }
}


async fn handle_socket(tcp_stream: &mut TcpStream) -> Result<()> {
    let mut target_stream = match TcpStream::connect("45.77.198.191:19132").await {
        Ok(stream) => stream,
        Err(err) => {
            error!("Failed when trying to connect to destination {}", err);
            return Ok(());
        }
    };

    let mut stream_duplex = DuplexTcpStream::join(tcp_stream, &mut target_stream, None);
    match stream_duplex.start().await {
        Ok(_) => info!("Successfully streamed data."),
        Err(_) => {}
    };

    Ok(())
}

async fn start_server() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:3333").await?;
    info!("server running on port 3333");
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let _ = handle_socket(&mut stream).await;
        });
    }
}

async fn start_udp_server() -> Result<()> {
    let listener = UdpSocket::bind("127.0.0.1:3337").await?;
    info!("server running in 3337");

    let mut buffer = BytesMut::with_capacity(1024 * 8);
    let (mut sender, mut receiver) = mpsc::channel::<BytesMut>(100);

    tokio::

    loop {
        let (bytes_read, target_addr) = listener.recv_from(&mut buffer).await?;
        debug!("received {} bytes from {}", bytes_read, target_addr);
        if 0 == bytes_read {
            return Ok(());
        }

        let _ = listener.send_to(&buffer[..bytes_read], &target_addr).await?;
        buffer.truncate(bytes_read);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let _ = start_udp_server().await;
}
