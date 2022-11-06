use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::debug;

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
    pub async fn start(&self, shutdown_signal: impl Future) -> Result<()> {
        match self.args.get_type() {
            AppCommandType::Listen(args) => {
                let cancellation_token = CancellationToken::new();
                let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel::<()>(1);

                let mut command = ListenCommand::new(Arc::new(args.clone()), shutdown_complete_tx.clone(), &cancellation_token);
                tokio::select! {
                    res = command.handle() => {
                        debug!("ListenCommand has been finished with {:?}", res);
                    },
                    _ = shutdown_signal => {
                        debug!("app received stop signal..");
                        cancellation_token.cancel();
                    },
                };

                // Drops shutdown_complete_tx for below instruction finish
                drop(shutdown_complete_tx);

                // waits for all internal threads/object that contains shutdown_complete_tx
                // to be dropped.
                let _ = shutdown_complete_rx.recv().await;
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
