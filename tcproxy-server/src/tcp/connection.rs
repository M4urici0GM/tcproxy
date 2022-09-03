use std::net::SocketAddr;

use bytes::BytesMut;
use tcproxy_core::TcpFrame;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::OwnedSemaphorePermit;
use tokio::task::JoinHandle;
use tracing::debug;
use uuid::Uuid;

use tcproxy_core::Result;

use crate::tcp::{RemoteConnectionReader, RemoteConnectionWriter};

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

    pub async fn start<T, U>(
        &mut self,
        reader: T,
        writer: U,
        receiver: Receiver<BytesMut>,
    ) -> Result<()>
    where
        T: AsyncRead + Send + Unpin + 'static,
        U: AsyncWrite + Send + Unpin + 'static,
    {
        tokio::select! {
            _ = self.spawn_reader(reader) => {},
            _ = self.spawn_writer(receiver, writer) => {},
        };

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
    where
        T: AsyncRead + Send + Unpin + 'static,
    {
        let mut reader = RemoteConnectionReader::new(self.connection_id, &self.client_sender);
        tokio::spawn(async move {
            let result = reader.start(connection_reader).await;
            match result {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }

    fn spawn_writer<T>(
        &self,
        receiver: Receiver<BytesMut>,
        connection_writer: T,
    ) -> JoinHandle<Result<()>>
    where
        T: AsyncWrite + Send + Unpin + 'static,
    {
        let mut writer = RemoteConnectionWriter::new(receiver, self.connection_addr);
        tokio::spawn(async move {
            let _ = writer.start(connection_writer).await;
            Ok(())
        })
    }

}
