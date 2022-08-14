use std::net::SocketAddr;

use bytes::BytesMut;
use tcproxy_core::TcpFrame;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Sender, Receiver};
use tokio::task::JoinHandle;
use tokio::sync::OwnedSemaphorePermit;
use uuid::Uuid;
use tracing::debug;

use tcproxy_core::Result;

use crate::tcp::{RemoteConnectionReader, RemoteConnectionWriter, DefaultStreamReader};

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

  pub fn spawn(mut self, stream: TcpStream, receiver: Receiver<BytesMut>) -> JoinHandle<Result<()>> {
      tokio::spawn(async move {
          let _ = self.start(stream, receiver).await;
          Ok(())
      })
  }

  async fn start(&mut self, stream: TcpStream, receiver: Receiver<BytesMut>) -> Result<()> {
      let (reader, writer) = stream.into_split();

      let reader = DefaultStreamReader::new(reader);

      let reader = RemoteConnectionReader::new(reader, self.connection_id, &self.client_sender);
      let writer = RemoteConnectionWriter::new(writer, receiver, self.connection_addr);

      tokio::select! {
          _ = reader.spawn() => {},
          _ = writer.spawn() => {},
      };

      debug!(
          "received none from connection {}, aborting",
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
}