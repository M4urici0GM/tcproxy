use bytes::BytesMut;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::Sender;
use tracing::{error, trace};
use uuid::Uuid;

use tcproxy_core::TcpFrame;
use tcproxy_core::{HostPacketData, Result};

pub struct RemoteConnectionReader {
    connection_id: Uuid,
    client_sender: Sender<TcpFrame>,
}

impl RemoteConnectionReader {
    pub fn new(connection_id: Uuid, sender: &Sender<TcpFrame>) -> Self {
        Self {
            connection_id,
            client_sender: sender.clone(),
        }
    }

    pub async fn start<T>(&mut self, mut reader: T) -> Result<()>
    where
        T: AsyncRead + Unpin,
    {
        let mut buffer = BytesMut::with_capacity(1024 * 4);
        loop {
            let bytes_read = match reader.read_buf(&mut buffer).await {
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

            trace!("read {} bytes from connection.", bytes_read);
            if 0 == bytes_read {
                trace!(
                    "reached end of stream from connection {}",
                    self.connection_id
                );
                break;
            }

            let frame = TcpFrame::HostPacket(HostPacketData::new(
                self.connection_id,
                buffer.split_to(bytes_read),
                bytes_read as u32,
            ));

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
    use bytes::{BufMut, BytesMut};
    use std::io::Cursor;

    use crate::tests::utils::generate_random_buffer;
    use tcproxy_core::TcpFrame;
    use tokio::sync::mpsc;
    use uuid::Uuid;

    use super::RemoteConnectionReader;

    #[tokio::test]
    async fn should_stop_if_read_zero_bytes() {
        // Arrange
        let uuid = Uuid::new_v4();
        let (sender, mut receiver) = mpsc::channel::<TcpFrame>(1);

        let empty_vec: Vec<u8> = vec![];
        let cursor = Cursor::new(&empty_vec[..]);
        let mut connection_reader = RemoteConnectionReader::new(uuid, &sender);

        // Act
        let result = connection_reader.start(cursor).await;

        // drops sender and connection_reader for receiver.recv() to resolve.
        drop(sender);
        drop(connection_reader);

        let receiver_result = receiver.recv().await;

        // Assert
        assert_eq!(true, result.is_ok());
        assert_eq!(true, receiver_result.is_none());
    }

    #[tokio::test]
    async fn should_read_correctly() {
        // Arrange
        let uuid = Uuid::new_v4();
        let expected_buff_size = 1024 * 6;
        let random_buffer = generate_random_buffer(expected_buff_size);
        let (sender, mut receiver) = mpsc::channel::<TcpFrame>(3);

        let mut reader = RemoteConnectionReader::new(uuid, &sender);
        let cursor = Cursor::new(&random_buffer[..]);

        // At this point stream is already closed, but underlying buffer still there for reading.
        let _ = reader.start(cursor).await;

        let mut final_buff = BytesMut::with_capacity(expected_buff_size as usize);
        for _ in 0..2 {
            if let Some(frame) = receiver.recv().await {
                match frame {
                    TcpFrame::HostPacket(data) => {
                        final_buff.put_slice(&data.buffer()[..]);
                    }
                    value => {
                        panic!("didnt expected {value}");
                    }
                }
            }
        }

        assert!(!final_buff.is_empty());
        assert_eq!(final_buff.len(), random_buffer.len());
        assert_eq!(&final_buff[..], &random_buffer[..]);
    }
}
