use tracing::info;
use tcproxy_core::Result;
use tcproxy_core::{TcpFrame, TransportWriter};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};


pub struct TcpFrameWriter {
  receiver: Receiver<TcpFrame>,
  writer: TransportWriter,
}


impl TcpFrameWriter {
  pub fn new(receiver: Receiver<TcpFrame>, writer: TransportWriter) -> Self {
      Self { receiver, writer }
  }

  pub fn spawn(mut self) -> JoinHandle<Result<()>> {
      tokio::spawn(async move {
          let _ = TcpFrameWriter::start(&mut self).await;
          Ok(())
      })
  }

  async fn start(&mut self) -> Result<()> {
      while let Some(msg) = self.receiver.recv().await {
          self.writer.send(msg).await?;
      }

      info!("Reached end of stream.");
      Ok(())
  }
}