use tracing::trace;

use crate::{Result, TcpFrame};
use tokio::io::{AsyncWrite, AsyncWriteExt};

/// represents TcpFrame transport writer.
/// writes TcpFrames into underlying buffer.
pub struct TransportWriter {
    writer: Box<dyn AsyncWrite + Send + Unpin>,
}

impl TransportWriter {
    pub(crate) fn new<T>(writer: T) -> Self
    where
        T: AsyncWrite + Send + Unpin + 'static,
    {
        Self {
            writer: Box::new(writer),
        }
    }

    /// writes TcpFrame into underlying tcp stream.
    pub async fn send(&mut self, frame: TcpFrame) -> Result<()> {
        let mut buffer = TcpFrame::to_buffer(&frame);
        let bytes_written = self.writer.write_buf(&mut buffer).await?;
        let _ = self.writer.flush().await;
        trace!("written {} bytes to socket.", bytes_written);

        Ok(())
    }
}
