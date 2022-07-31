use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::sync::Mutex;
use bytes::BytesMut;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::{info, debug, trace};
use uuid::Uuid;

use crate::{AppArguments, PortManager, Result};
use crate::proxy::{ProxyClient};
use crate::tcp::Listener;

#[derive(Debug)]
pub struct Server {
    args: Arc<AppArguments>,
    server_listener: Listener,
}

#[derive(Debug)]
pub struct ProxyClientState {
    db: Arc<Shared>,
}

#[derive(Debug)]
pub struct Shared {
    connections: Mutex<HashMap<Uuid, Sender<BytesMut>>>,
}

impl ProxyClientState {
    pub fn new() -> Self {
        Self {
            db: Arc::new(Shared {
                connections: Mutex::new(HashMap::new()),
            })
        }
    }

    pub fn insert_connection(&self, connection_id: Uuid, sender: Sender<BytesMut>) {
        let mut state = self.db.connections.lock().unwrap();
        state.insert(connection_id, sender);
        info!("{} connections..", state.len());
    }

    pub fn remove_connection(&self, connection_id: Uuid) {
        let mut state = self.db.connections.lock().unwrap();
        if !state.contains_key(&connection_id) {
            trace!("connection {} not found in state", connection_id);
            return;
        }

        info!("{} connections..", state.len());
        state.remove(&connection_id);
    }

    pub fn get_connection(&self, connection_id: Uuid) -> Option<Sender<BytesMut>> {
        let state = self.db.connections.lock().unwrap();
        if !state.contains_key(&connection_id) {
            trace!("connection {} not found in state", connection_id);
            return None;
        }

        info!("{} connections..", state.len());
        return Some(state.get(&connection_id).unwrap().clone());
    }
}

impl Server {
    pub fn new(args: AppArguments) -> Self {
        let ip = args.parse_ip().unwrap();
        let port = args.port();

        Self {
            args: Arc::new(args),
            server_listener: Listener { ip, port },
        }
    }

    async fn start(&self, cancellation_token: CancellationToken) -> Result<()> {
        let tcp_listener = self.server_listener.bind().await?;
        let available_proxies: Arc<Mutex<Vec<u16>>> = Arc::new(Mutex::new(Vec::new()));
        let port_range = self.args.parse_port_range()?;
        let listen_ip = self.args.parse_ip()?;

        let port_manager = PortManager {
            available_proxies: available_proxies.clone(),
            initial_port: port_range.0,
            final_port: port_range.1,
        };

        let proxy_state = Arc::new(ProxyClientState::new());
        while !cancellation_token.is_cancelled() {
            let (socket, addr) = self.server_listener.accept(&tcp_listener).await?;

            let cancellation_token = cancellation_token.child_token();
            let proxy_client = ProxyClient::new(listen_ip, addr, port_manager.clone(), proxy_state.clone());

            tokio::spawn(async move {
                match proxy_client.start_streaming(socket, cancellation_token).await {
                    Ok(_) => debug!("Socket {} has been closed gracefully.", addr),
                    Err(err) => debug!("Socket {} has been disconnected with error.. {}", addr, err)
                };
            });
        }

        Ok(())
    }


    pub async fn run(&self, shutdown_signal: impl Future) -> Result<()> {
        let cancellation_token = CancellationToken::new();
        tokio::select! {
            _ = self.start(cancellation_token.child_token()) => {

            },
            _ = shutdown_signal => {
                info!("server is being shut down.");
            }
        };

        cancellation_token.cancel();
        Ok(())
    }
}