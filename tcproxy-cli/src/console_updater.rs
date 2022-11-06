use emoji_printer::print_emojis;
use std::sync::Arc;
use tcproxy_core::Result;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use crate::{ClientState, ListenArgs};

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

    pub fn spawn(mut self, cancellation_token: &CancellationToken) -> JoinHandle<Result<()>> {
        let child_cancellation_token = cancellation_token.child_token();
        tokio::spawn(async move {
            Self::start(&mut self, child_cancellation_token).await;
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

    async fn start(&mut self, cancellation_token: CancellationToken) {
        while (self.receiver.recv().await).is_some() && !cancellation_token.is_cancelled() {
            if self.args.is_debug() {
                continue;
            }

            self.print_state();
        }

        self.clear();
    }
}
