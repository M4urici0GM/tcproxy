use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use tcproxy_core::transport::TransportWriter;
use tcproxy_core::{Result, TcpFrame};

pub struct ClientFrameWriter {
    pub(crate) receiver: Receiver<TcpFrame>,
    pub(crate) writer: TransportWriter,
    pub(crate) cancellation_token: CancellationToken,
}

impl ClientFrameWriter {
    pub fn start_writing(mut self) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let res = self.start().await;
            debug!("writer finished with {:?}", res);
            Ok(())
        })
    }

    async fn start(&mut self) -> Result<()> {
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
                    debug!("cancellation token from client cancelled");
                    break;
                },
            }
        }

        Ok(())
    }
}
