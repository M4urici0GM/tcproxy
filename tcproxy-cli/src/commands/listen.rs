use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Sender};

use tracing::{debug, info};

use tcproxy_core::framing::Reason;
use tcproxy_core::framing::{Authenticate, ClientConnected, GrantType, TokenAuthenticationArgs};
use tcproxy_core::{transport::TcpFrameTransport, AsyncCommand, Result, TcpFrame};

use crate::config::{AppConfig, AppContext};
use crate::server_addr::ServerAddr;
use crate::{
    ClientState, ConsoleUpdater, DefaultDirectoryResolver, ListenArgs, PingSender, Shutdown,
    TcpFrameReader, TcpFrameWriter,
};

use super::contexts::DirectoryResolver;

pub struct ListenCommand {
    args: Arc<ListenArgs>,
    app_cfg: Arc<AppConfig>,
    dir_resolver: DefaultDirectoryResolver,
    _shutdown_complete_tx: Sender<()>,
    _notify_shutdown: broadcast::Sender<()>,
}

impl ListenCommand {
    pub fn new(
        args: Arc<ListenArgs>,
        config: Arc<AppConfig>,
        dir_resolver: &DefaultDirectoryResolver,
        shutdown_complete_tx: Sender<()>,
        notify_shutdown: broadcast::Sender<()>,
    ) -> Self {
        Self {
            dir_resolver: dir_resolver.clone(),
            args: Arc::clone(&args),
            app_cfg: Arc::clone(&config),
            _notify_shutdown: notify_shutdown,
            _shutdown_complete_tx: shutdown_complete_tx,
        }
    }

    fn get_context(&self) -> Result<AppContext> {
        let context_name = self
            .args
            .app_context()
            .unwrap_or(self.app_cfg.default_context_str().to_string());

        match self.app_cfg.get_context(&context_name) {
            Some(ctx) => Ok(ctx),
            None => Err(format!("context {} was not found.", context_name).into()),
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

        
        let app_context = self.get_context()?;
        let addr = ServerAddr::try_from(app_context)?.to_socket_addr()?;
        let mut transport = TcpFrameTransport::connect(addr).await?;

        info!("Connected to server, trying handshake...");

        let (console_sender, console_receiver) = mpsc::channel::<i32>(10);
        let (sender, receiver) = mpsc::channel::<TcpFrame>(10000);
        let state = Arc::new(ClientState::new(&console_sender));
    
        do_handshake(&mut transport).await?;
        authenticate(&self.dir_resolver, &mut transport).await?;

        let (reader, writer) = transport.split();
        let ping_task = PingSender::new(
            &sender,
            &state,
            self.args.ping_interval(),
            &self._shutdown_complete_tx,
        );
        let console_task = ConsoleUpdater::new(
            console_receiver,
            &state,
            &self.args,
            &self._shutdown_complete_tx,
        );

        let receive_task = TcpFrameWriter::new(receiver, writer, &self._shutdown_complete_tx);
        let forward_task = TcpFrameReader::new(
            &sender,
            &state,
            &self.args,
            reader,
            &self._shutdown_complete_tx,
        );

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

async fn authenticate(
    dir_resolver: &DefaultDirectoryResolver,
    client: &mut TcpFrameTransport,
) -> Result<()> {
    let config_path = dir_resolver.get_config_file()?;
    let mut config = AppConfig::load(&config_path)?;

    let token = config.get_user_token().unwrap_or_default();
    let grant_type = GrantType::TOKEN(TokenAuthenticationArgs::new(&token));
    let authenticate_frame = TcpFrame::Authenticate(Authenticate::new(grant_type));

    match client.send_frame(&authenticate_frame).await? {
        TcpFrame::AuthenticateAck(data) => {
            debug!("authenticated successfully");
            debug!("trying to save user token into config file..");

            // Stores user token into local config file
            config.set_user_token(data.token());
            AppConfig::save_to_file(&config, &config_path)?;

            Ok(())
        }
        TcpFrame::Error(err) if *err.reason() == Reason::AuthenticationFailed => {
            Err("Authentication failed. Try logging again with tcproxy-cli login".into())
        }
        actual => {
            debug!("received invalid frame when doing handshake. received {} instead of ClientConnectedAck", actual);
            Err("Error while trying to communicate with server.".into())
        }
    }
}

async fn do_handshake(client: &mut TcpFrameTransport) -> Result<()> {
    let frame = TcpFrame::ClientConnected(ClientConnected);
    match client.send_frame(&frame).await? {
        TcpFrame::ClientConnectedAck(_) => Ok(()),
        actual => {
            debug!("received invalid frame when doing handshake. received {} instead of ClientConnectedAck", actual);
            Err("failed to do handshake with server.".into())
        }
    }
}
