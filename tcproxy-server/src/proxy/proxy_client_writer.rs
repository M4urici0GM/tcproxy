use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use tcproxy_core::transport::TransportWriter;
use tcproxy_core::{Result, TcpFrame};

pub struct ClientFrameWriter {
    receiver: Receiver<TcpFrame>,
    writer: TransportWriter,
    cancellation_token: CancellationToken,
}

impl ClientFrameWriter {
    pub fn new(
        receiver: Receiver<TcpFrame>,
        writer: TransportWriter,
        cancel_token: &CancellationToken,
    ) -> Self {
        Self {
            receiver,
            writer,
            cancellation_token: cancel_token.child_token(),
        }
    }

    pub fn start_writing(mut self) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let res = self.start().await;
            debug!("writer finished with {:?}", res);
            Ok(())
        })
    }

    async fn start(&mut self) -> Result<()> {
        while !self.cancellation_token.is_cancelled() {
            match self.receiver.recv().await {
                Some(frame) => {
                    self.writer.send(frame).await?;
                },
                None => {
                    debug!("received None from client channel");
                    break;
                }
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

}