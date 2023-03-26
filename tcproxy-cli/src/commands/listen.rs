use async_trait::async_trait;
use std::io::{stdout, Write};
use std::sync::Arc;
use tokio::net::TcpStream as TokioTcpStream;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::broadcast;

use tracing::{debug, error, info};

use tcproxy_core::tcp::TcpStream;
use tcproxy_core::{transport::TcpFrameTransport, AsyncCommand, Result, TcpFrame};
use tcproxy_core::framing::{Authenticate, ClientConnected, GrantType, TokenAuthenticationArgs, PasswordAuthArgs};
use tcproxy_core::transport::TransportReader;

use crate::{ClientState, ConsoleUpdater, ListenArgs, PingSender, Shutdown, TcpFrameReader, TcpFrameWriter};
use crate::config::{AppConfig, AppContext};


pub struct ListenCommand {
    args: Arc<ListenArgs>,
    app_cfg: Arc<AppConfig>,
    pub(crate) _shutdown_complete_tx: Sender<()>,
    pub(crate) _notify_shutdown: broadcast::Sender<()>,
}

impl ListenCommand {
    pub fn new(
        args: Arc<ListenArgs>,
        config: Arc<AppConfig>,
        shutdown_complete_tx: Sender<()>,
        notify_shutdown: broadcast::Sender<()>) -> Self
    {
        Self {
            args: Arc::clone(&args),
            app_cfg: Arc::clone(&config),
            _notify_shutdown: notify_shutdown,
            _shutdown_complete_tx: shutdown_complete_tx,
        }
    }

    fn get_context(&self) -> Result<AppContext> {
        let context_name = match self.args.app_context() {
            Some(ctx) => ctx,
            None => self.app_cfg.default_context().to_string(),
        };

        match self.app_cfg.get_context(&context_name) {
            Some(ctx) => Ok(ctx),
            None => {
                Err(format!("context {} was not found.", context_name).into())
            }
        }
    }

    /// connects to remote server.
    async fn connect(&self) -> Result<TcpStream> {
        let app_context = self.get_context()?;
        let server_addr = format!("{}:{}", app_context.host(), app_context.port());
        match TokioTcpStream::connect(server_addr).await {
            Ok(stream) => {
                debug!("Connected to server..");
                let socket_addr = stream.peer_addr().unwrap();
                Ok(TcpStream::new(stream, socket_addr))
            }
            Err(err) => {
                println!("{} {}", 124, 123);

                error!("Failed to connect to server. Check you network connection and try again.");
                Err(format!("Failed when connecting to server: {}", err).into())
            }
        }
    }
}

fn strip_newline(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix("\n"))
        .unwrap_or(input)
}

fn get_username_password() -> Result<PasswordAuthArgs> {
    let mut tries = 0;
    while tries < 3 {
        print!("Your email: ");
        stdout().flush()?;

        let mut username = String::default();
        let total_chars = std::io::stdin().read_line(&mut username)?;
        if 0 == total_chars {
            tries += 1;
            continue;
        }

        let password = rpassword::prompt_password("Your password: ")?;
        return Ok(PasswordAuthArgs::new(strip_newline(&username), &password, None));
    }

    return Err("Max tries reached.".into());
}

fn get_grant_type(app_cfg: &AppConfig) -> Result<GrantType> {
    match app_cfg.get_user_token() {
        Some(token) => {
            Ok(GrantType::TOKEN(TokenAuthenticationArgs::new(&token)))
        },
        None => Ok(GrantType::PASSWORD(get_username_password()?))
    }
}

#[async_trait]
impl AsyncCommand for ListenCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Result<()> {
        if self.args.is_debug() {
            tracing_subscriber::fmt::init();
        }

        info!("Trying to connect...");

        let connection = self.connect().await?;
        let (console_sender, console_receiver) = mpsc::channel::<i32>(10);
        let (sender, receiver) = mpsc::channel::<TcpFrame>(10000);
        let (mut reader, mut writer) = TcpFrameTransport::new(connection).split();

        info!("Connected to server, trying handshake...");

        let state = Arc::new(ClientState::new(&console_sender));

        writer.send(TcpFrame::ClientConnected(ClientConnected)).await?;
        let frame = match reader.next().await? {
            Some(f) => f,
            None => {
                debug!("received none. it means the server closed the connection.");
                return Err("failed to do handshake with server.".into());
            }
        };

        match frame {
            TcpFrame::ClientConnectedAck(_) => {},
            actual => {
                debug!("received invalid frame when doing handshake. received {} instead of ClientConnectedAck", actual);
                return Err("failed to do handshake with server.".into());
            }
        };

       let grant_type = get_grant_type(&self.app_cfg)?;

        writer.send(TcpFrame::Authenticate(Authenticate::new(grant_type))).await?;
        let frame = match reader.next().await? {
            Some(f) => f,
            None => {
                debug!("received none. it means the server closed the connection.");
                return Err("failed to do handshake with server.".into());
            }
        };

        match frame {
            TcpFrame::AuthenticateAck(data) => {
                debug!("received {} from authenticate frame", data);
            },
            actual => {
                debug!("received invalid frame when doing handshake. received {} instead of ClientConnectedAck", actual);
                return Err("You are not authenticated or your token expired. Please authenticate with\n tcproxy-cli auth login".into());
            }
        };


        let ping_task = PingSender::new(&sender, &state, self.args.ping_interval(), &self._shutdown_complete_tx);
        let console_task = ConsoleUpdater::new(console_receiver, &state, &self.args, &self._shutdown_complete_tx);
        let receive_task = TcpFrameWriter::new(receiver, writer, &self._shutdown_complete_tx);
        let forward_task = TcpFrameReader::new(&sender, &state, reader, &self.args, &self._shutdown_complete_tx);

        info!("Connected to server, spawning required tasks...");

        tokio::select! {
            res = console_task.spawn(Shutdown::new(self._notify_shutdown.subscribe())) => {
                debug!("console task finished. {:?}", res);
            }
            _ = receive_task.spawn(Shutdown::new(self._notify_shutdown.subscribe())) => {
                debug!("receive task finished.");
            },
            res = forward_task.spawn(Shutdown::new(self._notify_shutdown.subscribe())) => {
                debug!("forward to server task finished. {:?}", res);
            },
            _ = ping_task.spawn(Shutdown::new(self._notify_shutdown.subscribe())) => {
                debug!("ping task finished.");
            }
        }

        Ok(())
    }
}
