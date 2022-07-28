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
use clap::Parser;

use reverse_proxy::{Result, DuplexTcpStream, AppArguments, Server};

enum Message {
    Connected,
    DataBytes(BytesMut),
    Disconnected,
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
    let listener = TcpListener::bind("127.0.0.1:*").await?;
    info!("server running on port 3333");
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let _ = handle_socket(&mut stream).await;
        });
    }
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = AppArguments::parse();
    let server = Server::new(args);

    let _ = start_server().await;
}
