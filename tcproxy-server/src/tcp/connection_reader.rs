use tcproxy_core::tcp::StreamReader;
use tokio::sync::mpsc::Sender;
use tracing::{error, trace};

use tcproxy_core::{DataPacket, TcpFrame};
use tcproxy_core::{Result};

pub struct RemoteConnectionReader {
    connection_id: u32,
    client_sender: Sender<TcpFrame>,
}


impl RemoteConnectionReader {
    pub fn new(connection_id: &u32, sender: &Sender<TcpFrame>) -> Self {
        Self {
            connection_id: *connection_id,
            client_sender: sender.clone(),
        }
    }

    pub async fn start<T>(&mut self, mut reader: T) -> Result<()>
    where
        T: StreamReader,
    {
        while let Some(buffer) = reader.read().await? {
            let frame = TcpFrame::DataPacket(DataPacket::new(
                &self.connection_id,
                &buffer,
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
    use mockall::Sequence;
    use rand::random;
    use tokio::sync::mpsc;
    

    use crate::tests::utils::generate_random_buffer;
    use tcproxy_core::{TcpFrame, tcp::MockStreamReader};

    use super::*;

    #[tokio::test]
    async fn should_stop_if_read_none() {
        // Arrange
        let connection_id = random::<u32>();
        let (sender, mut receiver) = mpsc::channel::<TcpFrame>(1);
        let mut connection_reader = RemoteConnectionReader::new(&connection_id, &sender);
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
        assert!(result.is_ok());
        assert!(receiver_result.is_none());
    }

    #[tokio::test]
    async fn should_read_correctly() {
        // Arrange
        let connection_id = random::<u32>();
        let expected_buff_size = 1024 * 6;
        let random_buffer = generate_random_buffer(expected_buff_size);
        let (sender, mut receiver) = mpsc::channel::<TcpFrame>(3);

        let mut connection_reader = RemoteConnectionReader::new(&connection_id, &sender);

        let mut reader = MockStreamReader::new();
        let mut sequence = Sequence::new();
        let buff_clone = random_buffer.clone();
        reader.expect_read()
            .times(1)
            .returning(move || {
                Ok(Some(BytesMut::from(&buff_clone[..(buff_clone.len()/2)])))
            })
            .in_sequence(&mut sequence);

        let buff_clone = random_buffer.clone();
        reader.expect_read()
            .times(1)
            .returning(move || {
                Ok(Some(BytesMut::from(&buff_clone[(buff_clone.len()/2)..])))
            })
            .in_sequence(&mut sequence);

        reader.expect_read()
            .times(1)
            .returning(|| Ok(None))
            .in_sequence(&mut sequence);

        // At this point stream is already closed, but underlying buffer still there for reading.
        let _ = connection_reader.start(reader).await;

        let mut final_buff = BytesMut::with_capacity(expected_buff_size as usize);
        for _ in 0..2 {
            if let Some(frame) = receiver.recv().await {
                match frame {
                    TcpFrame::DataPacket(data) => {
                        final_buff.put_slice(data.buffer());
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
