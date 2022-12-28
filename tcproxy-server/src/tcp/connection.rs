use std::net::SocketAddr;

use tcproxy_core::tcp::{SocketConnection, DefaultStreamReader};
use tcproxy_core::{RemoteSocketDisconnected, TcpFrame};
use tokio::io::{AsyncWrite, AsyncRead};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::OwnedSemaphorePermit;
use tokio::task::JoinHandle;
use tracing::debug;

use tcproxy_core::Result;

    
use crate::tcp::{RemoteConnectionReader, RemoteConnectionWriter};

pub struct RemoteConnection {
    _permit: OwnedSemaphorePermit,
    connection_id: u32,
    connection_addr: SocketAddr,
    client_sender: Sender<TcpFrame>,
}

impl RemoteConnection {
    pub fn new(
        permit: OwnedSemaphorePermit,
        socket_addr: SocketAddr,
        id: &u32,
        client_sender: &Sender<TcpFrame>,
    ) -> Self {
        Self {
            _permit: permit,
            connection_id: *id,
            connection_addr: socket_addr,
            client_sender: client_sender.clone(),
        }
    }

    pub async fn start<T>(&mut self, connection: T, receiver: Receiver<Vec<u8>>) -> Result<()>
        where
            T: SocketConnection,
    {
        let (reader, writer) = connection.split();
        tokio::select! {
            _ = self.spawn_reader(reader) => {},
            _ = self.spawn_writer(receiver, writer) => {},
        }

        debug!(
            "received stop signal from connection {}. aborting..",
            self.connection_id
        );

        let _ = self
            .client_sender
            .send(TcpFrame::RemoteSocketDisconnected(RemoteSocketDisconnected::new(&self.connection_id)))
            .await;

        Ok(())
    }

    fn spawn_reader<T>(&self, reader: T) -> JoinHandle<Result<()>>
        where T: AsyncRead + Send + Unpin + 'static {
        let stream_reader = DefaultStreamReader::new(1024 * 8, reader);
        let mut reader = RemoteConnectionReader::new(&self.connection_id, &self.client_sender);
        tokio::spawn(async move {
            match reader.start(stream_reader).await {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }

    fn spawn_writer<T>(
        &self,
        receiver: Receiver<Vec<u8>>,
        connection_writer: T,
    ) -> JoinHandle<Result<()>> where T : AsyncWrite + Send + Unpin + 'static {
        let mut writer = RemoteConnectionWriter::new(receiver, self.connection_addr);
        tokio::spawn(async move {
            let _ = writer.start(connection_writer).await;
            Ok(())
        })
    }
}
