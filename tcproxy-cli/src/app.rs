use std::sync::Arc;

use tcproxy_core::Result;
use tcproxy_core::{AsyncCommand, Command};

use crate::contexts::CreateContextCommand;
use crate::ClientArgs;
use crate::ListenCommand;
use crate::{AppCommandType, ContextCommands};

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
            }
            AppCommandType::Context(args) => {
                println!("received config command");
                if let ContextCommands::Create(args) = args {
                    let mut command = CreateContextCommand::new(args);
                    let result = command.handle();
                    match result {
                        Ok(_) => println!("Hello"),
                        Err(err) => println!("{:?}", err),
                    }
                }

                // let mut command = ConfigCommand::new(args);
                // let _ = command.handle().await;
            }
        }
        Ok(())
    }
}
