use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;
use bytes::BytesMut;
use tokio::net::{TcpListener, TcpStream};
use tokio_codec::Framed;
use tracing::error;

use crate::{AppArguments, Result};

pub struct Server {
    args: Arc<AppArguments>,
}

pub struct ConnectionHandler {
    tcp_stream: TcpStream,
    buffer: BytesMut,
}

pub enum StreamFrame {
    Connected(),
    Disconnected(),

}

impl ConnectionHandler {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            tcp_stream: stream,
            buffer: BytesMut::with_capacity(1024 * 8),
        }
    }

    pub async fn start_streaming(&self) -> Result<()> {
        let framed = Framed::new(stream, )

        Ok(())
    }
}

impl Server {
    pub fn new(args: AppArguments) -> Self {
        Self {
            args: Arc::new(args),
        }
    }

    async fn bind(&self) -> Result<TcpListener> {
        let ip = self.args.parse_ip()?;
        match TcpListener::bind(SocketAddrV4::new(ip, self.args.port() as u16)).await {
            Ok(listener) => Ok(listener),
            Err(err) => {
                error!("Failed when binding to {}", ip);
                return Err(err.into());
            }
        }
    }

    async fn accept(&self, listener: &TcpListener) -> Result<(TcpStream, SocketAddr)> {
        let mut backoff = 1;
        loop {
            let result = listener.accept().await;
            if let Ok(result) = result {
                return Ok(result.unwrap());
            }

            if backoff > 64 {
                error!("Failed to accept new socket. aborting.. {}", err);
                return Err(err.into());
            }

            backoff *= 2;
        }
    }

    pub async fn listen(&self) -> Result<()> {
        let tcp_listener = self.bind().await?;
        loop {
            let (socket, addr) = self.accept(&tcp_listener).await?;
            let connection_handler = ConnectionHandler::new(socket);

            tokio::spawn(async move {
                let _ = connection_handler.start_streaming().await;
            });
        }
    }
}