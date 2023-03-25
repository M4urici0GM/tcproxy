use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use bytes::Bytes;
use directories::ProjectDirs;
use http_body_util::Full;
use hyper::{Method, Response};
use hyper::service::service_fn;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, broadcast};

use tracing::debug;

use tcproxy_core::Result;
use tcproxy_core::{AsyncCommand, Command};
use tcproxy_core::http::{StartChallengeRequest};

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
    // TODO: remove the self argument from DirectoryResolver
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
            AppCommandType::Login => {
                println!("hello world from login command");
                println!("starting authentication process...");

                let address = "127.0.0.1:0";
                let listener = TcpListener::bind(address).await.unwrap();
                let listener_port = listener.local_addr().unwrap().port();
                println!("started listening at {}", &listener_port);

                let nonce = rand::random::<u32>();
                let callback_uri = format!("http://127.0.0.1:{}", &listener_port);

                let request = StartChallengeRequest::new(&callback_uri, &nonce);
                let response = tcproxy_core::http::start_challenge(&request).await.unwrap();

                let open_url = format!("http://127.0.0.1:3000/signin?challenge={}", response.challenge_id());

                println!("Opening browser..");
                println!("If your browser doesn't automatically open, please copy and paste the following link in your browser:");
                println!("{}", &open_url);

                open::that(open_url)?;

                loop {
                    let (stream, _) = tokio::select! {
                        res = listener.accept() => {
                            println!("accepted socket..");
                            res?
                        },
                        _ = tokio::time::sleep(Duration::from_secs(300)) => {
                            println!("timeout reached.. exiting authentication challenge..");
                            break;
                        }
                    };

                    hyper::server::conn::http1::Builder::new()
                        .serve_connection(stream, service_fn(|req| async move {
                            let config_path = DefaultDirectoryResolver.get_config_file().unwrap();
                            let mut config = AppConfig::load(&config_path).unwrap();

                            if Method::GET != req.method() {
                                let response = Response::builder()
                                    .status(400)
                                    .header(hyper::header::CONNECTION, "close")
                                    .body(Full::new(Bytes::from("Invalid request")))
                                    .unwrap();

                                return Ok::<Response<Full<Bytes>>, Infallible>(response);
                            }

                            let query = match req.uri().query() {
                                Some(query) => query,
                                None => {
                                    let response = Response::builder()
                                        .status(400)
                                        .header(hyper::header::CONNECTION, "close")
                                        .body(Full::new(Bytes::from("Invalid request")))
                                        .unwrap();

                                    return Ok::<Response<Full<Bytes>>, Infallible>(response);
                                }
                            };

                            let params = url::form_urlencoded::parse(query.as_bytes())
                                .into_owned()
                                .collect::<HashMap<String, String>>();

                            return match params.get("token") {
                                Some(t) => {
                                    config.set_user_token(&t);
                                    AppConfig::save_to_file(&config, &config_path).unwrap();

                                    println!("authenticated successfully!");
                                    let response = Response::builder()
                                        .status(200)
                                        .header(hyper::header::CONNECTION, "close")
                                        .body(Full::new(Bytes::from("You can close this window now.")))
                                        .unwrap();

                                    Ok::<Response<Full<Bytes>>, Infallible>(response)
                                }
                                None => {
                                    let response = Response::builder()
                                        .status(400)
                                        .header(hyper::header::CONNECTION, "close")
                                        .body(Full::new(Bytes::from("Invalid request")))
                                        .unwrap();

                                    Ok::<Response<Full<Bytes>>, Infallible>(response)
                                }
                            };
                        }))
                        .await?;

                    break;
                }
            }
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
                        match res {
                            Ok(_) => {},
                            Err(err) => {
                                println!("error: {:?}", err);
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
                        CreateContextCommand::new(args, DefaultDirectoryResolver).handle()
                    }
                    ContextCommands::List => {
                        ListContextsCommand::new(DefaultDirectoryResolver).handle()
                    }
                    ContextCommands::SetDefault(args) => {
                        SetDefaultContextCommand::new(args, DefaultDirectoryResolver).handle()
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

                // let mut command = ConfigCommand::new(args);
                // let _ = command.handle().await;
            }
        }
        Ok(())
    }
}
