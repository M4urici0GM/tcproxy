use async_trait::async_trait;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpStream as TokioTcpStream;
use tokio::sync::mpsc::{self, Sender};
use tokio::sync::broadcast;

use tracing::{debug, error, info};

use tcproxy_core::tcp::TcpStream;
use tcproxy_core::{transport::TcpFrameTransport, AsyncCommand, Result, TcpFrame};
use tcproxy_core::framing::ClientConnected;

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
        let (reader, mut writer) = TcpFrameTransport::new(connection).split();

        info!("Connected to server, trying handshake...");

        let state = Arc::new(ClientState::new(&console_sender));

        writer.send(TcpFrame::ClientConnected(ClientConnected)).await?;

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
        ;

        Ok(())
    }
}
