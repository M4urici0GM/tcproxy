use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, error};

use crate::Result;

#[derive(Debug)]
pub struct Listener {
    pub(crate) ip: Ipv4Addr,
    pub(crate) port: u16,
}

impl Listener {
    pub fn create_socket_ip(ip: Ipv4Addr, port: u16) -> SocketAddrV4 {
        SocketAddrV4::new(ip, port)
    }

    pub async fn bind(&self) -> Result<TcpListener> {
        let ip = Listener::create_socket_ip(self.ip, self.port);
        match TcpListener::bind(ip).await {
            Ok(listener) => {
                info!("server running on port {}", self.port);
                Ok(listener)
            }
            Err(err) => {
                error!("Failed when binding to {}", ip);
                return Err(err.into());
            }
        }
    }

    pub async fn accept(&self, listener: &TcpListener) -> Result<(TcpStream, SocketAddr)> {
        let mut backoff = 1;
        loop {
            let result = listener.accept().await;
            if let Ok(result) = result {
                info!("New socket {} connected.", result.1);
                return Ok(result);
            }

            if backoff > 64 {
                let err = result.err().unwrap();
                error!("Failed to accept new socket. aborting.. {}", err);
                return Err(err.into());
            }

            backoff *= 2;
        }
    }
}