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