use tcproxy_core::{TcpFrame, Result};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use tracing::{trace,  debug};

pub struct ProxyClientStreamWriter {
    pub(crate) receiver: Receiver<TcpFrame>,
    pub(crate) writer: OwnedWriteHalf,
    pub(crate) cancellation_token: CancellationToken,
}

impl ProxyClientStreamWriter {
    pub async fn start_writing(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                frame = self.receiver.recv() => {
                    match frame {
                        Some(frame) => {
                            let mut buffer = frame.to_buffer();
                            let bytes_written = self.writer.write_buf(&mut buffer).await?;
                            trace!("written {} bytes to socket.", bytes_written);

                            self.writer.flush().await?;
                        },
                        None => {
                            debug!("received None from client channel");
                            break;
                        }
                    }
                },
                _ = self.cancellation_token.cancelled() => {
                    break;
                },
            }
        }

        Ok(())
    }
}