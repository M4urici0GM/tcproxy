use bytes::{BufMut, BytesMut};
use futures_util::{SinkExt, StreamExt};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tcproxy::Result;
use tracing::{error, info, debug};
use tcproxy::codec::{TcpFrame, TcpFrameCodec};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let tcp_connection = match TcpStream::connect("127.0.0.1:8080").await {
        Ok(stream) => stream,
        Err(err) => {
            return Err(format!("Failed when connecting to server: {}", err).into());
        },
    };


    let tcp_frame = TcpFrame::ClientConnected;
    let framed = Framed::new(tcp_connection, TcpFrameCodec);
    let (mut writer, mut reader) = framed.split();

    match writer.send(tcp_frame).await {
        Ok(_) => {
            info!("send initial frame to server..");
        },
        Err(err) => {
            error!("Failed sending initial frame to server: {}", err);
        },
    };


    loop {
        while let Some(msg) = reader.next().await {
            if msg.is_ok() {
                if let TcpFrame::DataPacket { connection_id, buffer } = msg.unwrap() {
                    debug!("Received new data packet from connection {}: {}", connection_id, String::from_utf8(buffer.to_vec())?);
                }
            }
        }
    }
}