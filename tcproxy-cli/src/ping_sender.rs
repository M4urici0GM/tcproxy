use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tcproxy_core::{Result, TcpFrame};
use tokio::sync::mpsc::Sender;
use tokio::time;
use tokio::{task::JoinHandle, time::Instant};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

use crate::{ClientState, Shutdown};

pub struct PingSender {
    interval: u64,
    state: Arc<ClientState>,
    sender: Sender<TcpFrame>,
    _shutdown_signal: Sender<()>
}

impl PingSender {
    pub fn new(
        sender: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
        interval: u8,
        shutdown_signal: &Sender<()>) -> Self {
        let interval = interval as u64;
        Self {
            interval,
            state: state.clone(),
            sender: sender.clone(),
            _shutdown_signal: shutdown_signal.clone()
        }
    }

    pub fn spawn(mut self, mut shutdown: Shutdown) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            tokio::select! {
                _ = PingSender::start(&mut self) => {
                    debug!("ping sender task stopped.");
                },
                _ = shutdown.recv() => {
                    debug!("received stop signal..");
                }
            };
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
                }
                Err(err) => {
                    error!("Failed to send ping. aborting. {}", err);
                    break;
                }
            };
        }

        Ok(())
    }
}
