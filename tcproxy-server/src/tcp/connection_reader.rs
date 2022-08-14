use async_trait::async_trait;
use bytes::BytesMut;
use mockall::automock;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tracing::{error, trace};
use uuid::Uuid;

use tcproxy_core::Result;
use tcproxy_core::TcpFrame;

pub struct RemoteConnectionReader {
    reader: Box<dyn StreamReader>,
    connection_id: Uuid,
    client_sender: Sender<TcpFrame>,
}

#[automock]
#[async_trait]
pub trait StreamReader: Sync + Send {
    async fn read_buf(&mut self, buffer: &mut BytesMut) -> Result<usize>;
}

pub struct DefaultStreamReader {
    inner: OwnedReadHalf,
}

#[async_trait]
impl StreamReader for DefaultStreamReader {
    async fn read_buf(&mut self, buffer: &mut BytesMut) -> Result<usize> {
        Ok(self.inner.read_buf(buffer).await?)
    }
}

impl DefaultStreamReader {
    pub fn new(inner: OwnedReadHalf) -> Self {
        Self { inner }
    }
}

impl RemoteConnectionReader {
    pub fn new<T>(reader: T, connection_id: Uuid, sender: &Sender<TcpFrame>) -> Self
    where
        T: StreamReader + 'static,
    {
        Self {
            reader: Box::new(reader),
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
        use crate::tcp::{MockStreamReader, RemoteConnectionReader};
        use crate::tests::utils::generate_random_buffer;
        use bytes::BufMut;
        use mockall::Sequence;
        use tcproxy_core::TcpFrame;
        use tokio::sync::mpsc;
        use uuid::Uuid;

        // Arrange
        let buffer_size = 1024;
        let mut seq = Sequence::new();
        let mut stream_reader = MockStreamReader::new();

        stream_reader
            .expect_read_buf()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |buff| {
                let random = generate_random_buffer(buffer_size);
                buff.put_slice(&random[..]);
                Ok(buffer_size as usize)
            });

        stream_reader
            .expect_read_buf()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(0));

        let connection_id = Uuid::new_v4();
        let (sender, mut receiver) = mpsc::channel::<TcpFrame>(1);
        let connection_reader = RemoteConnectionReader::new(stream_reader, connection_id, &sender);

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
