use emoji_printer::print_emojis;
use std::sync::Arc;
use tcproxy_core::Result;
use tokio::{sync::mpsc::Receiver, task::JoinHandle};

use crate::{ClientArgs, ClientState};

pub struct ConsoleUpdater {
    receiver: Receiver<i32>,
    state: Arc<ClientState>,
    args: Arc<ClientArgs>,
}

impl ConsoleUpdater {
    pub fn new(receiver: Receiver<i32>, state: &Arc<ClientState>, args: &Arc<ClientArgs>) -> Self {
        Self {
            receiver,
            args: args.clone(),
            state: state.clone(),
        }
    }

    pub fn spawn(mut self) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let _ = ConsoleUpdater::start(&mut self).await;
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

    async fn start(&mut self) {
        while let Some(_) = self.receiver.recv().await {
            if self.args.is_debug() {
                continue;
            }

            self.print_state();
        }

        self.clear();
    }
}
