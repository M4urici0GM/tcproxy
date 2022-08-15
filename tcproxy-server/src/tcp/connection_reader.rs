use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tracing::{error, trace};
use uuid::Uuid;

use tcproxy_core::Result;
use tcproxy_core::TcpFrame;

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
            let result = self.start().await;
            match result {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
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
                    return Err(err.into());
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
                    return Err(err.into());
                }
            }
        }

        trace!("received stop signal.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[cfg(test)]
    #[tokio::test]
    async fn should_read_frame_correctly() {
        use crate::extract_enum_value;
        use crate::tests::utils::generate_random_buffer;

        use super::*;
        use tokio::net::TcpListener;
        use tcproxy_core::TcpFrame;
        use tokio::net::TcpStream;
        use tokio::sync::mpsc;
        use tokio::io::AsyncWriteExt;
        use uuid::Uuid;

        // Arrange
        let buffer_size = 1024;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();

        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let mut connection = TcpStream::connect(addr).await.unwrap();
            let mut buffer = generate_random_buffer(1024);
            connection.write_buf(&mut buffer).await.unwrap();
        });
        
        let (stream, _) = listener.accept().await.unwrap();
        let (reader, _) = stream.into_split();

        let connection_id = Uuid::new_v4();
        let (sender, mut receiver) = mpsc::channel::<TcpFrame>(1);
        let connection_reader = RemoteConnectionReader::new(reader, connection_id, &sender);

        // Act
        let result = connection_reader.spawn().await;
        let frame = receiver.recv().await;

        assert!(result.is_ok());
        assert!(frame.is_some());

        let frame = frame.unwrap();
        assert!(matches!(frame, TcpFrame::DataPacketHost { .. }));

        let (_, size, id) = extract_enum_value!(frame, TcpFrame::DataPacketHost { buffer, buffer_size, connection_id} => (buffer, buffer_size, connection_id));

        assert_eq!(size as i32, buffer_size);
        assert_eq!(id, connection_id);
    }
}
