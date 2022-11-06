use tcproxy_core::transport::TransportWriter;
use tcproxy_core::Result;
use tcproxy_core::TcpFrame;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::{info, debug};
use crate::Shutdown;

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

    pub fn spawn(mut self, mut shutdown: Shutdown) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let _ = TcpFrameWriter::start(&mut self, shutdown).await;
            Ok(())
        })
    }

    async fn start(&mut self, mut shutdown: Shutdown) -> Result<()> {
        while !shutdown.is_shutdown() {
            let msg = tokio::select! {
                res = self.receiver.recv() => res,
                _ = shutdown.recv() => {
                    debug!("received stop signal from cancellation token");
                    return Ok(())
                }
            };

            let msg = match msg {
                Some(msg) => msg,
                None => break,
            };

            self.writer.send(msg).await?;
        }

        info!("Reached end of stream.");
        Ok(())
    }
}
