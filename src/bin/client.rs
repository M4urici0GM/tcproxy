use bytes::{Buf, BytesMut};
use futures_util::StreamExt;
use tokio::task::JoinHandle;
use std::collections::HashMap;
use std::io::Cursor;
use std::pin::Pin;
use std::sync::Arc;
use tcproxy::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedWriteHalf, OwnedReadHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender, Receiver};
use tokio::time::{self, Duration, Instant};
use tokio_stream::StreamMap;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use uuid::Uuid;

use tcproxy::codec::{FrameError, TcpFrame};

type TcpFrameStream = Pin<Box<dyn tokio_stream::Stream<Item = TcpFrame> + Send>>;


async fn parse_frame(buffer: &mut BytesMut) -> Result<Option<TcpFrame>> {
    let mut cursor = Cursor::new(&buffer[..]);
    match TcpFrame::check(&mut cursor) {
        Ok(_) => {
            let position = cursor.position() as usize;
            cursor.set_position(0);

            let frame = TcpFrame::parse(&mut cursor)?;
            buffer.advance(position);

            Ok(Some(frame))
        }
        Err(FrameError::Incomplete) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

async fn receive_frame(reader: &mut OwnedReadHalf, buffer: &mut BytesMut) -> Result<Option<TcpFrame>> {
    loop {
        if let Some(frame) = parse_frame(buffer).await? {
            return Ok(Some(frame));
        }

        if 0 == reader.read_buf(buffer).await? {
            if buffer.is_empty() {
                debug!("received 0 bytes from client, and buffer is empty.");
                return Ok(None);
            }

            return Err("connection reset by peer.".into());
        }
    }
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

    let (mut reader, mut writer) = tcp_connection.into_split();


    let mut buff = TcpFrame::ClientConnected.to_buffer();
    let _ = writer.write_buf(&mut buff).await;

    loop {
        let mut buffer = BytesMut::with_capacity(1024 * 8);
        let maybe_frame = receive_frame(&mut reader, &mut buffer).await?;

        let frame = match maybe_frame {
            Some(f) => f,
            None => {
                break;
            }
        };
        
        match frame {
            TcpFrame::ClientConnectedAck { port } => {
                info!("Remote proxy listening in {}", port);
            },
            TcpFrame::DataPacketHost { connection_id, buffer: _ } => {
                debug!("received DataPacketHost from host");
                let buff = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"test\": \"hello world\"}";
                let mut buff = TcpFrame::DataPacketClient { connection_id, buffer: BytesMut::from(&buff[..]) }.to_buffer();
                let _ = writer.write_buf(&mut buff).await;

                let mut buff = TcpFrame::LocalClientDisconnected { connection_id }.to_buffer();
                let _ = writer.write_buf(&mut buff).await;
            },
            aaa => {
                debug!("received {} frame", aaa);
            },
        }
    }


    Ok(())
}
