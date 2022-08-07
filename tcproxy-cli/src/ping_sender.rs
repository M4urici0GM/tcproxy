use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use tcproxy_core::{TcpFrame, Result};
use tokio::{task::JoinHandle, time::Instant};
use tokio::sync::mpsc::Sender;
use tokio::time;
use tracing::{debug, error};

use crate::ClientState;

pub struct PingSender {
  interval: u64,
  state: Arc<ClientState>,
  sender: Sender<TcpFrame>,
}

impl PingSender {
  pub fn new(sender: &Sender<TcpFrame>, state: &Arc<ClientState>, interval: Option<u64>) -> Self {
      let interval = match interval {
          Some(i) => i,
          None => 30,
      };

      Self {
          interval,
          state: state.clone(),
          sender: sender.clone(),
      }
  }

  pub fn spawn(mut self) -> JoinHandle<Result<()>> {
      tokio::spawn(async move {
          let _ = PingSender::start(&mut self).await;
          Ok(())
      })
  }

  async fn start(&mut self) -> Result<()> {
      loop {
          debug!("Waiting for next ping to occur");
          time::sleep_until(Instant::now() + Duration::from_secs(self.interval)).await;
          match self.sender.send(TcpFrame::Ping).await {
              Ok(_) => {
                let time = Utc::now();
                self.state.update_last_sent_ping(time);

                debug!("Sent ping frame..");
              },
              Err(err) => {
                  error!("Failed to send ping. aborting. {}", err);
                  break;
              }
          };
      }

      Ok(())
  }
}
