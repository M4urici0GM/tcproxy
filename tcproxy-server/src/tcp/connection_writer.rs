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

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use tokio::{net::{TcpListener, TcpStream}, io::AsyncReadExt, sync::mpsc};

    use crate::tests::utils::generate_random_buffer;

    #[tokio::test]
    async fn should_write_buffer_correctly() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

        let addr = listener.local_addr().unwrap();
        let thread_handler = tokio::spawn(async move {
            let mut connection = TcpStream::connect(addr).await.unwrap();

            let mut buff = BytesMut::with_capacity(1024 * 4);
            let bytes_read = connection.read_buf(&mut buff).await.unwrap();
            (bytes_read, buff.split_to(bytes_read))
        });


        let buffer_size = 1024;
        let buffer = generate_random_buffer(buffer_size);
        let (sender, receiver) = mpsc::channel::<BytesMut>(1);
        let (stream, addr) = listener.accept().await.unwrap();
        let (_, writer) = stream.into_split();

        sender.send(BytesMut::from(&buffer[..])).await.unwrap();
        RemoteConnectionWriter::new(writer, receiver, addr).spawn();

        let (bytes_read, buff) = thread_handler.await.unwrap();
        assert_eq!(bytes_read, buffer_size as usize);
        assert_eq!(&buff[..], &buffer[..]);
    }
}