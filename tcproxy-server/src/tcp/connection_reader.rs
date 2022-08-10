use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::task::JoinHandle;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;
use tracing::{trace, error};
 
use tcproxy_core::TcpFrame;
use tcproxy_core::Result;

pub struct RemoteConnectionReader {
  reader: OwnedReadHalf,
  connection_id: Uuid,
  client_sender: Sender<TcpFrame>,
}

impl RemoteConnectionReader {
  pub fn new(reader: OwnedReadHalf, connection_id: Uuid, sender: &Sender<TcpFrame>) -> Self {
      Self {
          reader,
          connection_id,
          client_sender: sender.clone(),
      }
  }

  pub fn spawn(mut self) -> JoinHandle<Result<()>> {
      tokio::spawn(async move {
          let _ = self.start().await;
          Ok(())
      })
  }

  async fn start(&mut self) -> Result<()> {
      loop {
          let mut buffer = BytesMut::with_capacity(1024 * 8);
          let bytes_read = match self.reader.read_buf(&mut buffer).await {
              Ok(read) => read,
              Err(err) => {
                  trace!(
                      "failed to read from connection {}: {}",
                      self.connection_id,
                      err
                  );
                  break;
              }
          };

          if 0 == bytes_read {
              trace!(
                  "reached end of stream from connection {}",
                  self.connection_id
              );
              break;
          }

          buffer.truncate(bytes_read);
          let buffer = BytesMut::from(&buffer[..]);
          let frame = TcpFrame::DataPacketHost {
              connection_id: self.connection_id,
              buffer,
              buffer_size: bytes_read as u32,
          };

          match self.client_sender.send(frame).await {
              Ok(_) => {}
              Err(err) => {
                  error!("failed to send frame to client. {}", err);
                  break;
              }
          }
      }

      trace!("received stop signal.");
      Ok(())
  }
}