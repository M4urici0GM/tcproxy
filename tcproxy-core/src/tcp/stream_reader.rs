use mockall::automock;
use async_trait::async_trait;
use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt};
use tracing::trace;

use crate::Result;

#[automock]
#[async_trait]
pub trait StreamReader: Send {
    async fn read(&mut self) -> Result<Option<BytesMut>>;
}

pub struct DefaultStreamReader {
    buffer: BytesMut,
    stream: Box<dyn AsyncRead + Unpin + Send>,

}

impl DefaultStreamReader {
    pub fn new<T>(buffer_size: usize, stream: T) -> Self
        where T: AsyncRead + Unpin + Send + 'static {
        Self {
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
                        "failed to read from connection: {}",
                        err
                    );
                return Err(err.into());
            }
        };

        trace!("read {} bytes from connection.", bytes_read);
        if 0 == bytes_read {
            trace!("reached end of stream from remote socket.");
            return Ok(None);
        }

        Ok(Some(self.buffer.split_to(bytes_read)))
    }
}

#[cfg(test)]
pub mod test {
    use std::io::ErrorKind;
    use tokio_test::io::Builder;
    use crate::tcp::{DefaultStreamReader, StreamReader};
    use crate::test_util::generate_random_buffer;

    #[tokio::test]
    pub async fn test() {
        let expected_err = std::io::Error::new(ErrorKind::BrokenPipe, "failed when reading");
        let mock = Builder::new()
            .read_error(expected_err)
            .build();

        let mut reader = DefaultStreamReader::new(1024, mock);

        // Act
        let result = reader.read().await;

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    pub async fn should_read_correctly() {
        // Arrange
        let expected_buffer = generate_random_buffer(1024);
        let reader_mock = Builder::new()
            .read(&expected_buffer[..])
            .build();

        let mut reader = DefaultStreamReader::new(1024, reader_mock);

        // Act
        let buffer = reader.read().await.unwrap();

        // Assert
        assert!(buffer.is_some());
        assert_eq!(&buffer.unwrap()[..], &expected_buffer[..]);
    }

    #[tokio::test]
    pub async fn should_return_none_if_buffer_is_empty() {
        // Arrange
        let expected_buffer = Vec::new();
        let reader_mock = Builder::new()
            .read(&expected_buffer[..])
            .build();

        let mut reader = DefaultStreamReader::new(1024, reader_mock);

        // Act
        let buffer = reader.read().await.unwrap();

        // Assert
        assert!(buffer.is_none());
    }
}