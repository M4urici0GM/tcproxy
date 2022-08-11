use std::fmt::Debug;
use std::future::Future;
use std::net::{SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{info, error, debug};
use tcproxy_core::Result;
use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};

use crate::{AppArguments, ProxyState};
use crate::proxy::Connection;
use crate::tcp::ListenerUtils;

#[derive(Debug)]
pub struct Server {
    args: Arc<AppArguments>,
    server_listener: Box<dyn Listener>,
}


#[async_trait]
pub trait Listener: Debug + Sync + Send {
    fn listen_ip(&self) -> SocketAddr;
    async fn accept(&mut self) -> Result<(TcpStream, SocketAddr)>;
}

#[derive(Debug)]
pub struct DefaultListener {
    listener: TcpListener,
}

impl DefaultListener {
    pub async fn bind(ip: SocketAddr) -> Result<Self> {
        let listener = TcpListener::bind(ip).await?;
        Ok(Self {
            listener,
        })
    }
}

#[async_trait]
impl Listener for DefaultListener {
    fn listen_ip(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    async fn accept(&mut self) -> Result<(TcpStream, SocketAddr)> {
        let mut backoff = 1;
        loop {
            match self.listener.accept().await {
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


impl Server {
    pub fn new(args: AppArguments, listener: Box<dyn Listener>) -> Self {
        let ip = args.parse_ip().unwrap();
        let port = args.port();

        Self {
            args: Arc::new(args),
            server_listener: listener,
        }
    }

    pub async fn run(&mut self, shutdown_signal: impl Future) -> Result<()> {
        let cancellation_token = CancellationToken::new();
        tokio::select! {
            _ = self.start(cancellation_token.child_token()) => {},
            _ = shutdown_signal => {
                info!("server is being shut down.");
                cancellation_token.cancel();
            }
        };

        Ok(())
    }

    pub fn get_listen_ip(&self) -> SocketAddr {
        self.server_listener.listen_ip()
    }

    async fn start(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        let port_range = self.args.parse_port_range()?;
        let listen_ip = self.args.parse_ip()?;

        while !cancellation_token.is_cancelled() {
            let (socket, addr) = self.server_listener.accept().await?;

            let proxy_state = Arc::new(ProxyState::new(&port_range));
            let cancellation_token = cancellation_token.child_token();
            let mut proxy_client = Connection::new(listen_ip, proxy_state.clone());

            tokio::spawn(async move {
                match proxy_client.start_streaming(socket, cancellation_token).await {
                    Ok(_) => debug!("Socket {} has been closed gracefully.", addr),
                    Err(err) => debug!("Socket {} has been disconnected with error.. {}", addr, err)
                };
            });
        }

        Ok(())
    }
}