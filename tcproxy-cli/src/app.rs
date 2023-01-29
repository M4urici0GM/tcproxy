use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use directories::ProjectDirs;
use tokio::sync::{mpsc, broadcast};

use tracing::debug;

use tcproxy_core::Result;
use tcproxy_core::{AsyncCommand, Command};

use crate::{ClientArgs};
use crate::commands::ListenCommand;
use crate::commands::contexts::{CreateContextCommand, DirectoryResolver, ListContextsCommand, SetDefaultContextCommand};
use crate::{AppCommandType, ContextCommands};
use crate::config::AppConfig;

/// represents main app logic.
pub struct App {
    args: Arc<ClientArgs>,
}

pub struct DefaultDirectoryResolver;

impl DefaultDirectoryResolver {
    pub fn new() -> Self {
        Self {}
    }

    fn get_config_dir() -> Result<ProjectDirs> {
        let project_dir = ProjectDirs::from("", "m4urici0gm", "tcproxy");
        match project_dir {
            Some(dir) => Ok(dir),
            None => Err("Couldnt access config folder".into()),
        }
    }
}

impl DirectoryResolver for DefaultDirectoryResolver {
    fn get_config_folder(&self) -> Result<PathBuf> {
        let project_dir = DefaultDirectoryResolver::get_config_dir()?;
        let config_dir = project_dir.config_dir();

        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir)?;
        }

        Ok(PathBuf::from(&config_dir))
    }

    fn get_config_file(&self) -> Result<PathBuf> {
        let mut base_path = self.get_config_folder()?;
        base_path.push("config.yaml");

        Ok(base_path)
    }
}

impl App {
    pub fn new(args: ClientArgs) -> Self {
        Self {
            args: Arc::new(args),
        }
    }

    /// does initial handshake and start listening for remote connections.
    pub async fn start(&self, shutdown_signal: impl Future) -> Result<()> {
        let directory_resolver = DefaultDirectoryResolver::new();
        let config_path = directory_resolver.get_config_file()?;
        let config = AppConfig::load(&config_path)?;

        match self.args.get_type() {
            AppCommandType::Listen(args) => {
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
                        CreateContextCommand::new(args, DefaultDirectoryResolver).handle()
                    },
                    ContextCommands::List => {
                        ListContextsCommand::new(DefaultDirectoryResolver).handle()
                    },
                    ContextCommands::SetDefault(args) => {
                        SetDefaultContextCommand::new(args, DefaultDirectoryResolver).handle()
                    },
                    _ => {
                        todo!()
                    }
                };

                match result {
                    Ok(_) => {},
                    Err(err) => {
                        println!("Failed when running command: {}", err);
                    }
                }

                // let mut command = ConfigCommand::new(args);
                // let _ = command.handle().await;
            }
        }
        Ok(())
    }
}
