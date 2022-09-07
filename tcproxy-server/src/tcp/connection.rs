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

pub trait SocketConnection: Sync + Send {
    fn split(
        self,
    ) -> (
        Box<dyn AsyncRead + Send + Unpin>,
        Box<dyn AsyncWrite + Send + Unpin>,
    );
}

pub struct Socket {
    pub inner: TcpStream,
}

impl SocketConnection for Socket {
    fn split(
        self,
    ) -> (
        Box<dyn AsyncRead + Send + Unpin>,
        Box<dyn AsyncWrite + Send + Unpin>,
    ) {
        let (reader, writer) = self.inner.into_split();

        (Box::new(reader), Box::new(writer))
    }
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

    fn spawn_reader(
        &self,
        connection_reader: Box<dyn AsyncRead + Send + Unpin>,
    ) -> JoinHandle<Result<()>> {
        let mut reader = RemoteConnectionReader::new(self.connection_id, &self.client_sender);
        tokio::spawn(async move {
            let result = reader.start(connection_reader).await;
            match result {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }

    fn spawn_writer(
        &self,
        receiver: Receiver<BytesMut>,
        connection_writer: Box<dyn AsyncWrite + Send + Unpin>,
    ) -> JoinHandle<Result<()>> {
        let mut writer = RemoteConnectionWriter::new(receiver, self.connection_addr);
        tokio::spawn(async move {
            let _ = writer.start(connection_writer).await;
            Ok(())
        })
    }
}
