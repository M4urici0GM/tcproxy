use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use tcproxy_core::Result;
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, error, debug};

#[derive(Debug)]
pub struct ListenerUtils {
    pub(crate) socket_addr: SocketAddr,
}

impl ListenerUtils {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { socket_addr: SocketAddr::new(ip, port) }
    }

    pub fn listen_ip(&self) -> SocketAddr {
        self.socket_addr
    }

    pub async fn bind(&self) -> Result<TcpListener> {
        match TcpListener::bind(self.socket_addr).await {
            Ok(listener) => {
                info!("server running on {}", self.socket_addr);
                Ok(listener)
            }
            Err(err) => {
                error!("Failed when binding to {}", self.socket_addr);
                return Err(err.into());
            }
        }
    }

    pub async fn accept(&self, listener: &TcpListener) -> Result<(TcpStream, SocketAddr)> {
        let mut backoff = 1;
        loop {
            match listener.accept().await {
                Ok(result) => {
                    debug!("New socket {} connected.", result.1);
                    return Ok(result);
                },
                Err(err) => {
                    error!("Failed to accept new socket. retrying.. {}", err);
                    if backoff > 64 {
                        error!("Failed to accept new socket. aborting.. {}", err);
                        return Err(err.into());
                    }
        
                    backoff *= 2;
                }
            };
        }
    }
}