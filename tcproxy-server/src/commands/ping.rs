use tokio::sync::mpsc::Sender;
use async_trait::async_trait;
use tcproxy_core::{Result, TcpFrame, AsyncCommand};

pub struct PingCommand {
  sender: Sender<TcpFrame>,
}

impl PingCommand {
  pub fn new(sender: &Sender<TcpFrame>) -> Self {
    Self {
      sender: sender.clone(),
    }
  }
}

#[async_trait]
impl AsyncCommand for PingCommand {
  type Output = Result<()>;

  async fn handle(&mut self) -> Self::Output {
      let _ = self.sender.send(TcpFrame::Pong).await;
      Ok(())
  }
}

