use bytes::BytesMut;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc::Sender;
use tracing::{error, trace};
use uuid::Uuid;

use tcproxy_core::Result;
use tcproxy_core::TcpFrame;

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
        let mut buffer = BytesMut::with_capacity(1024 * 8);
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
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use mockall::{mock, Sequence};
    use tcproxy_core::TcpFrame;
    use tokio::{io::{AsyncRead}, sync::mpsc};
    use uuid::Uuid;

    use crate::{tests::utils::generate_random_buffer, extract_enum_value};

    use super::RemoteConnectionReader;

    mock! {
        pub Reader {}

        impl AsyncRead for Reader {
            fn poll_read<'a, 'b>(self: Pin<&mut Self>, ctx: &mut Context<'a>, buf: &mut tokio::io::ReadBuf<'b>) -> Poll<Result<(), std::io::Error>>;
        }
    }

    #[tokio::test]
    async fn should_stop_if_read_zero_bytes() {
        // Arrange
        let uuid = Uuid::new_v4();
        let (sender, mut receiver) = mpsc::channel::<TcpFrame>(1);

        let mut mock_reader = MockReader::new();
        mock_reader.expect_poll_read()
            .returning(|_, buff| {
                println!("{:?}", buff);
                Poll::Ready(Ok(()))
            });

        let mut connection_reader = RemoteConnectionReader::new(uuid, &sender);

        // Act
        let result = connection_reader.start(mock_reader).await;

        // drops sender and connection_reader for receiver.recv() to resolve.
        drop(sender);
        drop(connection_reader);

        println!("HERE");
        let receiver_result = receiver.recv().await;

        // Assert
        assert_eq!(true, result.is_ok());
        assert_eq!(true, receiver_result.is_none());
    }

    #[tokio::test]
    async fn should_read_correctly() {
        // Arrange
        let uuid = Uuid::new_v4();
        let random_buffer = generate_random_buffer(1024 * 2);
        let (sender, mut receiver) = mpsc::channel::<TcpFrame>(1);
        
        let mut seq = Sequence::new();
        let mut mock_reader = MockReader::new();
        let buff_clone = random_buffer.clone();

        mock_reader.expect_poll_read()
            .times(1)
            .in_sequence(&mut seq)
            .returning(move |_, buff| {
                buff.put_slice(&buff_clone[..]);
                Poll::Ready(Ok(()))
            });

        mock_reader.expect_poll_read()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_, _| Poll::Ready(Ok(())));

        let mut connection_reader = RemoteConnectionReader::new(uuid, &sender);

        // Act
        let result = connection_reader.start(mock_reader).await;
        let (buff, buff_size) = extract_enum_value!(
            receiver.recv().await.unwrap(),
            TcpFrame::DataPacketHost { connection_id: _, buffer, buffer_size } => (buffer, buffer_size)
        );

        // Assert
        assert!(buff_size > 0);
        assert_eq!(true, result.is_ok());
        assert_eq!(buff_size as usize, random_buffer.len());
        assert_eq!(&random_buffer[..], &buff[..]);

    }
}
