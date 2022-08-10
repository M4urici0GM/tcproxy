use tokio::sync::mpsc::Sender;
use async_trait::async_trait;
use tcproxy_core::{Result, TcpFrame, Command};

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
impl Command for PingCommand {
  type Output = ();
  async fn handle(&mut self) -> Result<()> {
      let _ = self.sender.send(TcpFrame::Pong).await;
      Ok(())
  }
}

