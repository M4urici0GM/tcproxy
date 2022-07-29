use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use bytes::BytesMut;
use futures_util::SinkExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc, broadcast};
use tokio::sync::mpsc::Sender;
use tokio_util::codec::Framed;
use tracing::{error, info, debug};
use futures_util::stream::StreamExt;
use rand::Rng;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;
use tokio::sync;


use crate::{AppArguments, Result};
use crate::codec::{TcpFrame, TcpFrameCodec};

#[derive(Debug)]
pub struct Server {
    args: Arc<AppArguments>,
}

pub struct ClientConnection {
    tcp_stream: TcpStream,
    buffer: BytesMut,
    listen_ip: Ipv4Addr,
    available_port_range: (u16, u16),
    available_proxies: Arc<Mutex<Vec<u16>>>,
}

#[derive(Debug)]
pub(crate) struct Shutdown {
    /// `true` if the shutdown signal has been received
    shutdown: bool,

    /// The receive half of the channel used to listen for shutdown.
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    /// Create a new `Shutdown` backed by the given `broadcast::Receiver`.
    pub(crate) fn new(notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown {
            shutdown: false,
            notify,
        }
    }

    /// Returns `true` if the shutdown signal has been received.
    pub(crate) fn is_shutdown(&self) -> bool {
        self.shutdown
    }

    /// Receive the shutdown notice, waiting if necessary.
    pub(crate) async fn recv(&mut self) {
        // If the shutdown signal has already been received, then return
        // immediately.
        if self.shutdown {
            return;
        }

        // Cannot receive a "lag error" as only one value is ever sent.
        let _ = self.notify.recv().await;

        // Remember that the signal has been received.
        self.shutdown = true;
    }
}

/// Represents Client connected to server
impl ClientConnection {
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
            buffer: BytesMut::with_capacity(1024 * 8),
        }
    }

    pub async fn start_streaming(&mut self) -> Result<()> {
        let transport = Framed::new(&mut self.tcp_stream, TcpFrameCodec);
        let (mut transport_writer, mut transport_reader) = transport.split();
        let (notify_shutdown, _) = broadcast::channel::<()>(1);

        let connections: Arc<Mutex<HashMap<Uuid, Sender<BytesMut>>>> = Arc::new(Mutex::new(HashMap::new()));
        let (main_sender, mut receiver) = mpsc::channel::<TcpFrame>(100);

        let available_proxies = self.available_proxies.clone();
        let (initial_port, last_port) = self.available_port_range;
        let listen_ip = self.listen_ip;

        let notify_shutdown_clone = notify_shutdown.clone();
        let task1 = async move {
            loop {
                let received = transport_reader.next().await;
                if let None = received {
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
                        let mut socket_shutdown = Shutdown::new(notify_shutdown_clone.subscribe());

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
                        tokio::spawn(async move {
                            let server_task = async move {
                                let ip = SocketAddrV4::new(listen_ip, random_port);
                                let listener = match TcpListener::bind(ip).await {
                                    Ok(socket) => {
                                        info!("spawning new listener in {}", ip);
                                        socket
                                    }
                                    Err(err) => {
                                        error!("Failed when listening to port {}, {}", random_port, err);
                                        return;
                                    }
                                };

                                async fn start_loop(
                                    aa: &TcpListener,
                                    available_connections: Arc<Mutex<HashMap<Uuid, Sender<BytesMut>>>>,
                                    host_sender: Sender<TcpFrame>,
                                    listening_ip: &SocketAddrV4) -> Result<()> {
                                    loop {
                                        let available_connections = available_connections.clone();
                                        let host_sender = host_sender.clone();
                                        let (mut connection, addr) = aa.accept().await?;
                                        info!("received new socket in listener {}", listening_ip);

                                        tokio::spawn(async move {
                                            let (mut reader, mut writer) = connection.split();

                                            let connection_id = Uuid::new_v4();

                                            let (sender, mut receiver) = mpsc::channel::<BytesMut>(100);
                                            let mut available_connections = available_connections.lock().await;

                                            available_connections.insert(connection_id, sender.clone());
                                            drop(available_connections);


                                            let task1 = async move {
                                                while let Some(mut msg) = receiver.recv().await {
                                                    let _ = writer.write_buf(&mut msg).await;
                                                }
                                            };

                                            let task2 = async move {
                                                let mut buffer = BytesMut::with_capacity(1024 * 8);
                                                loop {
                                                    let bytes_read = match reader.read_buf(&mut buffer).await {
                                                        Ok(size) => size,
                                                        Err(err) => {
                                                            error!("Failed when reading from connection {}: {}", connection_id, err);
                                                            return;
                                                        }
                                                    };

                                                    if 0 == bytes_read {
                                                        debug!("reached end of stream");
                                                        return;
                                                    }

                                                    buffer.truncate(bytes_read);
                                                    let tcp_frame = TcpFrame::DataPacket {
                                                        connection_id,
                                                        buffer: buffer.clone(),
                                                    };

                                                    match host_sender.send(tcp_frame).await {
                                                        Ok(_) => {}
                                                        Err(err) => {
                                                            error!("failed when sending frame to main thread loop. connection {}: {}", connection_id, err);
                                                            return;
                                                        }
                                                    };
                                                }
                                            };

                                            tokio::select! {
                                                _ = task1 => {},
                                                _ = task2 => {},
                                            }
                                        });
                                    }
                                }

                                let _ = start_loop(&listener, connections.clone(), host_sender, &ip).await;
                            };

                            tokio::select! {
                                _ = server_task => {},
                                _ = socket_shutdown.recv() => {
                                    info!("client disconnected. closing server...");
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
                            error!("error sending packet to {}", connection_id);
                            return;
                        }
                    };
                }
            }
        };

        tokio::select! {
            _ = task1 => {},
            _ = task2 => {},
        }

        drop(notify_shutdown);

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
            Ok(listener) => {
                info!("server running on port {}", self.args.port());
                Ok(listener)
            }
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

    pub async fn listen(&self) -> Result<()> {
        let tcp_listener = self.bind().await?;
        let available_proxies: Arc<Mutex<Vec<u16>>> = Arc::new(Mutex::new(Vec::new()));
        let port_range = self.args.parse_port_range()?;
        let listen_ip = self.args.parse_ip()?;

        loop {
            let (socket, addr) = self.accept(&tcp_listener).await?;
            let mut connection_handler = ClientConnection::new(socket, port_range, listen_ip, available_proxies.clone());

            tokio::spawn(async move {
                match connection_handler.start_streaming().await {
                    Ok(_) => {
                        debug!("Socket {} has been closed gracefully.", addr);
                    }
                    Err(err) => {
                        debug!("Socket {} has been disconnected with error.. {}", addr, err);
                    }
                };
            });
        }
    }
}