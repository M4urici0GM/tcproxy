use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::OwnedSemaphorePermit;
use tracing::debug;

use tcproxy_core::framing::SocketDisconnected;
use tcproxy_core::tcp::{DefaultStreamReader, SocketConnection};
use tcproxy_core::Result;
use tcproxy_core::TcpFrame;

use crate::tcp::{RemoteConnectionReader, RemoteConnectionWriter};

pub struct RemoteConnection {
    connection_id: u32,
    client_sender: Sender<TcpFrame>,
    _permit: OwnedSemaphorePermit,
}

impl RemoteConnection {
    pub fn new(id: &u32, permit: OwnedSemaphorePermit, client_sender: &Sender<TcpFrame>) -> Self {
        Self {
            _permit: permit,
            connection_id: *id,
            client_sender: client_sender.clone(),
        }
    }

    pub async fn start<T>(self, connection: T, receiver: Receiver<Vec<u8>>) -> Result<()>
    where
        T: SocketConnection,
    {
        let connection_addr = connection.addr();
        let (reader, writer) = connection.split();

        let stream_reader = DefaultStreamReader::new(1024 * 8, reader);
        let mut reader =
            RemoteConnectionReader::new(&self.connection_id, &self.client_sender, stream_reader);
        let mut writer = RemoteConnectionWriter::new(receiver, connection_addr, writer);

        tokio::spawn(async move {
            let reader_task = tokio::spawn(async move {
                let _ = reader.start().await;
            });

            let writer_task = tokio::spawn(async move {
                let _ = writer.start().await;
            });

            tokio::select! {
                _ = reader_task => {},
                _ = writer_task => {},
            };

            debug!(
                "received stop signal from connection {}. aborting..",
                self.connection_id
            );
            let frame = TcpFrame::SocketDisconnected(SocketDisconnected::new(&self.connection_id));
            let _ = self.client_sender.send(frame).await;
        });

        Ok(())
    }
}
