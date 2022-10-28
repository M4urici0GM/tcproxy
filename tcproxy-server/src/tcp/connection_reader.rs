use std::fs::read;
use async_trait::async_trait;
use bytes::BytesMut;
use mockall::automock;
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

#[automock]
#[async_trait]
pub trait StreamReader {
    async fn read(&mut self) -> Result<Option<BytesMut>>;
}

pub struct DefaultStreamReader {
    buffer: BytesMut,
    connection_id: Uuid,
    stream: Box<dyn AsyncRead + Unpin + Send>,

}

impl DefaultStreamReader {
    pub fn new<T>(connection_id: Uuid, buffer_size: usize, stream: T) -> Self
        where T: AsyncRead + Unpin + Send + 'static {
        Self {
            connection_id,
            buffer: BytesMut::with_capacity(buffer_size),
            stream: Box::new(stream),
        }
    }
}

#[async_trait]
impl StreamReader for DefaultStreamReader {
    async fn read(&mut self) -> Result<Option<BytesMut>> {
        let bytes_read = match self.stream.read_buf(&mut self.buffer).await {
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
            return Ok(None);
        }

        Ok(Some(self.buffer.split_to(bytes_read)))
    }
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
        T: StreamReader,
    {
        while let Some(buffer) = reader.read().await? {
            let buffer_size = buffer.len() as u32;
            let frame = TcpFrame::HostPacket(HostPacketData::new(
                self.connection_id,
                buffer,
                buffer_size,
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

    use super::*;

    #[tokio::test]
    async fn should_stop_if_read_none() {
        // Arrange
        let uuid = Uuid::new_v4();
        let (sender, mut receiver) = mpsc::channel::<TcpFrame>(1);
        let mut connection_reader = RemoteConnectionReader::new(uuid, &sender);
        let mut reader = MockStreamReader::new();

        reader.expect_read()
            .returning(|| {
                Ok(None)
            });


        // Act
        let result = connection_reader.start(reader).await;

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

        let mut connection_reader = RemoteConnectionReader::new(uuid, &sender);

        let mut reader = MockStreamReader::new();
        let buff_clone = random_buffer.clone();
        reader.expect_read()
            .returning(move || {
                Ok(Some(BytesMut::from(&buff_clone[..])))
            });

        // At this point stream is already closed, but underlying buffer still there for reading.
        let _ = connection_reader.start(reader).await;

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
