use std::net::SocketAddr;
use bytes::BytesMut;
use tokio::{sync::mpsc::Receiver, net::tcp::OwnedWriteHalf};
use tokio::task::JoinHandle;
use tokio::io::AsyncWriteExt;
use tracing::{error, trace};

use tcproxy_core::Result;

pub struct RemoteConnectionWriter {
  writer: OwnedWriteHalf,
  connection_addr: SocketAddr,
  receiver: Receiver<BytesMut>,
}

impl RemoteConnectionWriter {
  pub fn new(
      writer: OwnedWriteHalf,
      receiver: Receiver<BytesMut>,
      connection_addr: SocketAddr,
  ) -> Self {
      Self {
          writer,
          connection_addr,
          receiver,
      }
  }

  pub fn spawn(mut self) -> JoinHandle<Result<()>> {
      tokio::spawn(async move {
          let _ = self.start().await;
          Ok(())
      })
  }

  async fn start(&mut self) -> Result<()> {
      while let Some(mut buffer) = self.receiver.recv().await {
          let mut buffer = buffer.split();
          match self.writer.write_buf(&mut buffer).await {
              Ok(written) => {
                  trace!("written {} bytes to {}", written, self.connection_addr)
              }
              Err(err) => {
                  error!("failed to write into {}: {}", self.connection_addr, err);
                  break;
              }
          };

          let _ = self.writer.flush().await;
      }

      self.receiver.close();
      Ok(())
  }
}
