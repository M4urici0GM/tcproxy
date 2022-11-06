use tcproxy_core::transport::TransportWriter;
use tcproxy_core::Result;
use tcproxy_core::TcpFrame;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub struct TcpFrameWriter {
    receiver: Receiver<TcpFrame>,
    writer: TransportWriter,
    _shutdown_complete_tx: Sender<()>
}

impl TcpFrameWriter {
    pub fn new(receiver: Receiver<TcpFrame>, writer: TransportWriter, shutdown_complete_tx: &Sender<()>) -> Self {
        Self {
            receiver,
            writer,
            _shutdown_complete_tx: shutdown_complete_tx.clone()
        }
    }

    pub fn spawn(mut self, cancellation_token: &CancellationToken) -> JoinHandle<Result<()>> {
        let cancellation_token = cancellation_token.child_token();
        tokio::spawn(async move {
            let _ = TcpFrameWriter::start(&mut self, cancellation_token).await;
            Ok(())
        })
    }

    async fn start(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        while !cancellation_token.is_cancelled() {
            let msg = match self.receiver.recv().await {
                Some(msg) => msg,
                None => break,
            };

            self.writer.send(msg).await?;
        }

        info!("Reached end of stream.");
        Ok(())
    }
}
