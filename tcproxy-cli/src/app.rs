use std::sync::Arc;

use tcproxy_core::Command;
use tcproxy_core::Result;

use crate::ClientArgs;
use crate::AppCommandType;
use crate::ListenCommand;

/// represents main app logic.
pub struct App {
    args: Arc<ClientArgs>,
}

impl App {
    pub fn new(args: ClientArgs) -> Self {
        Self {
            args: Arc::new(args),
        }
    }

    /// does initial handshake and start listening for remote connections.
    pub async fn start(&self) -> Result<()> {

        match self.args.get_type() {
            AppCommandType::Listen(args) => {
                let mut command = ListenCommand::new(Arc::new(args.clone()));
                let _ = command.handle().await;
            },
            AppCommandType::Context(args) => {
                println!("received config command");
                // let mut command = ConfigCommand::new(args);
                // let _ = command.handle().await;
            }
        }
        Ok(())
    }
}
