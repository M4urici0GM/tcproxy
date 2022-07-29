use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc};
use bytes::BytesMut;
use uuid::Uuid;
use tracing::{debug, error};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Sender;

use crate::Result;
use crate::codec::TcpFrame;

pub struct TcpConnection {
    pub(crate) connection: TcpStream,
    pub(crate) connection_id: Uuid,
    pub(crate) _connection_addr: SocketAddr,
    pub(crate) host_sender: Sender<TcpFrame>,
    pub(crate) available_connections: Arc<Mutex<HashMap<Uuid, Sender<BytesMut>>>>,
}

impl TcpConnection {
    async fn read_from_socket<T>(connection_id: Uuid, host_sender: &Sender<TcpFrame>, reader: &mut T) -> Result<()>
        where  T: AsyncRead + Unpin {
        let mut buffer = BytesMut::with_capacity(1024 * 8);
        loop {
            let bytes_read = match reader.read_buf(&mut buffer).await {
                Ok(size) => size,
                Err(err) => {
                    error!("Failed when reading from connection {}: {}", connection_id, err);
                    return Err("Error when reading from connection.".into());
                }
            };

            if 0 == bytes_read {
                debug!("reached end of stream");
                return Ok(());
            }

            buffer.truncate(bytes_read);
            let tcp_frame = TcpFrame::DataPacket { connection_id, buffer: buffer.clone() };

            match host_sender.send(tcp_frame).await {
                Ok(_) => {}
                Err(err) => return Err(format!("failed when sending frame to main thread loop. connection {}: {}", connection_id, err).into())
            };
        }
    }

    pub(crate) async fn handle_connection(&mut self) -> Result<()> {
        let (mut reader, mut writer) = self.connection.split();

        let (sender, mut receiver) = mpsc::channel::<BytesMut>(100);
        let mut available_connections = self.available_connections.lock().await;

        available_connections.insert(self.connection_id, sender.clone());
        drop(available_connections);


        let task1 = async move {
            while let Some(mut msg) = receiver.recv().await {
                let _ = writer.write_buf(&mut msg).await;
            }
        };

        let task2 = TcpConnection::read_from_socket(self.connection_id, &self.host_sender, &mut reader);

        tokio::select! {
            _ = task1 => {},
            _ = task2 => {},
        };

        Ok(())
    }
}