use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use tcproxy_core::{TcpFrame, Result};
use tcproxy_core::transport::TransportWriter;

pub struct ProxyClientStreamWriter {
    pub(crate) receiver: Receiver<TcpFrame>,
    pub(crate) writer: TransportWriter,
    pub(crate) cancellation_token: CancellationToken,
}

impl ProxyClientStreamWriter {
    pub async fn start_writing(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                frame = self.receiver.recv() => {
                    match frame {
                        Some(frame) => {
                            self.writer.send(frame).await?;
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