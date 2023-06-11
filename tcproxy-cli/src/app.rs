use crate::commands::contexts::{
    CreateContextCommand, ListContextsCommand, SetDefaultContextCommand,
};
use crate::commands::{ListenCommand, LoginCommand};
use crate::{config::{directory_resolver, self}, AppCommandType, ClientArgs, ContextCommands};
use std::future::Future;
use std::sync::Arc;
use tcproxy_core::{AsyncCommand, Command, Result};
use tokio::sync::{broadcast, mpsc};
use tracing::debug;

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
        let directory_resolver = directory_resolver::load()?;
        let config = config::load(&directory_resolver)?;

        match self.args.get_type() {
            AppCommandType::Login(args) => {
                let mut command = LoginCommand::new(args, &config);
                match command.handle().await {
                    Ok(_) => {
                        println!("authenticated successfully");
                    }
                    Err(err) => {
                        println!("unexpected error when trying to authenticate: {}", err);
                    }
                }
            }
            AppCommandType::Listen(args) => {
                // TODO: abstract this into a better way.
                // used to notify running threads that stop signal was received.
                let (notify_shutdown, _) = broadcast::channel::<()>(1);

                // used to wait for all threads to finish before closing the program..
                let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel::<()>(1);

                let mut command = ListenCommand::new(
                    Arc::new(args.clone()),
                    Arc::new(config.clone()),
                    shutdown_complete_tx,
                    notify_shutdown,
                );

                tokio::select! {
                    res = command.handle() => {
                        debug!("ListenCommand has been finished with {:?}", res);
                        match res {
                            Ok(_) => {},
                            Err(err) => {
                                println!("{}", err);
                            }
                        }
                    },
                    _ = shutdown_signal => {
                        debug!("app received stop signal..");
                    },
                };

                drop(command);

                // waits for all internal threads/object that contains shutdown_complete_tx
                // to be dropped.
                let _ = shutdown_complete_rx.recv().await;
            }
            AppCommandType::Context(args) => {
                let result = match args {
                    ContextCommands::Create(args) => {
                        CreateContextCommand::new(args, &config).handle()
                    }
                    ContextCommands::List => ListContextsCommand::new(&config).handle(),
                    ContextCommands::SetDefault(args) => {
                        SetDefaultContextCommand::new(args, &config).handle()
                    }
                    _ => {
                        todo!()
                    }
                };

                match result {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Failed when running command: {}", err);
                    }
                }
            }
        }

        config::save_to_disk(&config, &directory_resolver)?;
        Ok(())
    }
}
