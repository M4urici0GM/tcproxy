use emoji_printer::print_emojis;
use std::sync::Arc;
use tcproxy_core::Result;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::{ClientState, ListenArgs, Shutdown};

pub struct ConsoleUpdater {
    receiver: Receiver<i32>,
    state: Arc<ClientState>,
    args: Arc<ListenArgs>,
    _shutdown_complete_tx: Sender<()>
}

impl ConsoleUpdater {
    pub fn new(receiver: Receiver<i32>, state: &Arc<ClientState>, args: &Arc<ListenArgs>, shutdown_complete_signal: &Sender<()>) -> Self {
        Self {
            receiver,
            args: args.clone(),
            state: state.clone(),
            _shutdown_complete_tx: shutdown_complete_signal.clone()
        }
    }

    pub fn spawn(mut self, mut shutdown: Shutdown) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            Self::start(&mut self, shutdown).await;
            Ok(())
        })
    }

    fn clear(&self) {
        print!("\x1B[2J\x1B[1;1H");
    }

    fn print_state(&self) {
        self.clear();
        let state = self.state.get_console_status();
        let ip = self.args.parse_socket_addr();

        let msg = print_emojis(&format!(
            ":rocket: Server running at {} -> {}\n:dizzy: Ping: {:.2}ms\n:anchor: Connections: {}",
            state.remote_ip, ip, state.ping, state.connections
        ));
        println!("{}", msg);
    }

    async fn start(&mut self, mut shutdown: Shutdown) {
        loop {
            tokio::select! {
                res = self.receiver.recv() => {
                    if let None = res {
                        break;
                    }

                    if self.args.is_debug() {
                        continue;
                    }

                    self.print_state();
                }
                _ = shutdown.recv() => {
                    debug!("received stop signal.");
                    return;
                }
            }

        }

        self.clear();
    }
}
