use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::{Arc};
use bytes::BytesMut;
use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use tracing::{info, error, debug};

use crate::Result;
use crate::codec::{TcpFrameCodec, TcpFrame};
use crate::proxy::proxy_server::ProxyServer;

pub struct ProxyClient {
    tcp_stream: TcpStream,
    listen_ip: Ipv4Addr,
    available_port_range: (u16, u16),
    available_proxies: Arc<Mutex<Vec<u16>>>,
}

impl ProxyClient {
    pub fn new(
        stream: TcpStream,
        port_range: (u16, u16),
        listen_ip: Ipv4Addr,
        available_proxies: Arc<Mutex<Vec<u16>>>) -> Self {
        Self {
            tcp_stream: stream,
            available_proxies,
            listen_ip,
            available_port_range: port_range,
        }
    }

    pub async fn start_streaming(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        let transport = Framed::new(&mut self.tcp_stream, TcpFrameCodec);
        let (mut transport_writer, mut transport_reader) = transport.split();

        let connections: Arc<Mutex<HashMap<Uuid, Sender<BytesMut>>>> = Arc::new(Mutex::new(HashMap::new()));
        let (main_sender, mut receiver) = mpsc::channel::<TcpFrame>(100);

        let available_proxies = self.available_proxies.clone();
        let (initial_port, last_port) = self.available_port_range;
        let listen_ip = self.listen_ip;

        let client_cancellation_token = CancellationToken::new();
        let client_cancellation_clone = client_cancellation_token.child_token();
        let task1 = async move {
            while !client_cancellation_clone.is_cancelled() {
                let received = transport_reader.next().await;
                if received.is_none() {
                    debug!("No frame received from client. Aborting.");
                    return;
                }

                let frame = match received.unwrap() {
                    Ok(frame) => frame,
                    Err(err) => {
                        error!("Error when parsing frame. {}", err);
                        return;
                    }
                };

                match frame {
                    TcpFrame::DataPacket { buffer, connection_id } => {
                        let all_connections = connections.clone();
                        let connections_lock = all_connections.lock().await;

                        if !connections_lock.contains_key(&connection_id) {
                            error!("connection id {} not found.", connection_id);
                            return;
                        }

                        let connection_sender = connections_lock.get(&connection_id).unwrap();
                        match connection_sender.send(buffer).await {
                            Ok(_) => {}
                            Err(err) => {
                                error!("failed when sending data to connection {}: {}", connection_id, err);
                                return;
                            }
                        };
                    }
                    TcpFrame::ClientConnected => {
                        let mut tries = 0;
                        let mut mutex_lock = available_proxies.lock().await;

                        let mut rng = rand::thread_rng();
                        let mut random_port = rng.gen_range(initial_port..last_port);


                        while mutex_lock.contains(&random_port) {
                            tries += 1;
                            random_port = rng.gen_range(initial_port..last_port);

                            if tries == mutex_lock.len() {
                                error!("could not accept more connections, all ports used.");
                                return;
                            }
                        }

                        mutex_lock.push(random_port);
                        drop(mutex_lock);
                        drop(rng);

                        let connections = connections.clone();
                        let host_sender = main_sender.clone();
                        let cancellation_token = client_cancellation_clone.child_token();
                        tokio::spawn(async move {
                            let proxy_server = ProxyServer {
                                host_sender,
                                available_connections: connections,
                                listen_ip,
                                port: random_port,
                            };

                            tokio::select! {
                                _ = proxy_server.listen() => {},
                                _ = cancellation_token.cancelled() => {
                                    info!("client disconnected. closing server {}:{}...", listen_ip, random_port);
                                }
                            }
                        });
                    }
                    _ => {}
                }
            }
        };

        let task2 = async move {
            while let Some(msg) = receiver.recv().await {
                if let TcpFrame::DataPacket { connection_id, buffer } = msg {
                    let tcp_frame = TcpFrame::DataPacket { connection_id, buffer };
                    match transport_writer.send(tcp_frame).await {
                        Ok(_) => {
                            info!("send new packet to {}", connection_id);
                        }
                        Err(err) => {
                            error!("error sending packet to {}: {}", connection_id, err);
                            return;
                        }
                    };
                }
            }
        };

        tokio::select! {
            _ = task1 => {},
            _ = task2 => {},
            _ = cancellation_token.cancelled() => {}
        }

        client_cancellation_token.cancel();

        Ok(())
    }
}