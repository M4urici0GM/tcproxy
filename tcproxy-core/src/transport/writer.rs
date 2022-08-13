use tracing::trace;

use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};
use crate::{TcpFrame, Result};

/// represents TcpFrame transport writer.
/// writes TcpFrames into underlying buffer.
pub struct TransportWriter {
    writer: OwnedWriteHalf,
}

impl TransportWriter {
    pub(crate) fn new(writer: OwnedWriteHalf) -> Self {
        Self {
            writer,
        }
    }

    /// writes TcpFrame into underlying tcp stream.
    pub async fn send(&mut self, frame: TcpFrame) -> Result<()> {
        let mut buffer = frame.to_buffer();
        let bytes_written = self.writer.write_buf(&mut buffer).await?;
        let _ = self.writer.flush().await;
        trace!("written {} bytes to socket.", bytes_written);

        Ok(())
    }
}
