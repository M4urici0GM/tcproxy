use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::{Arc};
use bytes::BytesMut;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use uuid::Uuid;
use tracing::info;

use crate::Result;
use crate::codec::TcpFrame;
use crate::tcp::{Listener, TcpConnection};

pub struct ProxyServer {
    pub(crate) listen_ip: Ipv4Addr,
    pub(crate) port: u16,
    pub(crate) host_sender: Sender<TcpFrame>,
    pub(crate) available_connections: Arc<Mutex<HashMap<Uuid, Sender<BytesMut>>>>,
}

impl ProxyServer {
    pub async fn listen(&self) -> Result<()> {
        let listener = Listener { port: self.port, ip: self.listen_ip };
        let tcp_socket = listener.bind().await.unwrap();

        loop {
            let (connection, connection_addr) = tcp_socket.accept().await?;
            let host_sender = self.host_sender.clone();
            let available_connections = self.available_connections.clone();

            info!("received new socket in listener {}", Listener::create_socket_ip(self.listen_ip, self.port));
            tokio::spawn(async move {
                let connection_id = Uuid::new_v4(); 
                let mut incoming_tcp_connection = TcpConnection {
                    host_sender: host_sender.clone(),
                    available_connections,
                    connection,
                    _connection_addr: connection_addr,
                    connection_id,
                };

                let _ = incoming_tcp_connection.handle_connection().await;
                host_sender.send(TcpFrame::RemoteSocketDisconnected { connection_id }).await;
            });
        }

    }
}