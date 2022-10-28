use std::net::SocketAddr;

use bytes::BytesMut;
use tcproxy_core::tcp::SocketConnection;
use tcproxy_core::TcpFrame;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::OwnedSemaphorePermit;
use tokio::task::JoinHandle;
use tracing::debug;
use uuid::Uuid;

use tcproxy_core::Result;

use crate::tcp::{DefaultStreamReader, RemoteConnectionReader, RemoteConnectionWriter, StreamReader};

pub struct RemoteConnection {
    _permit: OwnedSemaphorePermit,
    connection_id: Uuid,
    connection_addr: SocketAddr,
    client_sender: Sender<TcpFrame>,
}

impl RemoteConnection {
    pub fn new(
        permit: OwnedSemaphorePermit,
        socket_addr: SocketAddr,
        id: Uuid,
        client_sender: &Sender<TcpFrame>,
    ) -> Self {
        Self {
            _permit: permit,
            connection_id: id,
            connection_addr: socket_addr,
            client_sender: client_sender.clone(),
        }
    }

    pub async fn start<T>(&mut self, connection: T, receiver: Receiver<BytesMut>) -> Result<()>
        where
            T: SocketConnection,
    {
        let (reader, writer) = connection.split();
        let stream_reader = DefaultStreamReader::new(self.connection_id, 1024 * 8, reader);
        tokio::select! {
            _ = self.spawn_reader(stream_reader) => {},
            _ = self.spawn_writer(receiver, writer) => {},
        }

        debug!(
            "received stop signal from connection {}. aborting..",
            self.connection_id
        );

        let _ = self
            .client_sender
            .send(TcpFrame::RemoteSocketDisconnected {
                connection_id: self.connection_id,
            })
            .await;

        Ok(())
    }

    fn spawn_reader<T>(&self, connection_reader: T) -> JoinHandle<Result<()>>
        where T: StreamReader + Send + 'static {
        let mut reader = RemoteConnectionReader::new(self.connection_id, &self.client_sender);
        tokio::spawn(async move {
            match reader.start(connection_reader).await {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }

    fn spawn_writer<T>(
        &self,
        receiver: Receiver<BytesMut>,
        connection_writer: T,
    ) -> JoinHandle<Result<()>> where T : AsyncWrite + Send + Unpin + 'static {
        let mut writer = RemoteConnectionWriter::new(receiver, self.connection_addr);
        tokio::spawn(async move {
            let _ = writer.start(connection_writer).await;
            Ok(())
        })
    }
}
