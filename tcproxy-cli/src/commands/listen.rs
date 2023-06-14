use async_trait::async_trait;
use std::sync::Arc;
use tcproxy_core::auth::token_handler::AuthToken;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Sender};

use tracing::{debug, error, info};

use tcproxy_core::framing::Reason;
use tcproxy_core::framing::{Authenticate, ClientConnected, GrantType, TokenAuthenticationArgs};
use tcproxy_core::{transport::TcpFrameTransport, AsyncCommand, Result, TcpFrame};

use crate::config::{AppContext, Config};
use crate::server_addr::ServerAddr;
use crate::{
    ClientState, ConsoleUpdater, ListenArgs, PingSender, Shutdown, TcpFrameReader, TcpFrameWriter,
};

pub struct ListenCommand {
    args: Arc<ListenArgs>,
    config: Arc<Config>,
    _shutdown_complete_tx: Sender<()>,
    _notify_shutdown: broadcast::Sender<()>,
}

impl ListenCommand {
    pub fn new(
        args: Arc<ListenArgs>,
        config: Arc<Config>,
        shutdown_complete_tx: Sender<()>,
        notify_shutdown: broadcast::Sender<()>,
    ) -> Self {
        Self {
            config: config.clone(),
            args: Arc::clone(&args),
            _notify_shutdown: notify_shutdown,
            _shutdown_complete_tx: shutdown_complete_tx,
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

        let app_context = get_context(&self.args, &self.config)?;
        let mut transport = get_transport(&app_context).await?;

        let (console_sender, console_receiver) = mpsc::channel::<i32>(10);
        let (sender, receiver) = mpsc::channel::<TcpFrame>(10000);
        let state = Arc::new(ClientState::new(&console_sender));

        let token = get_token(&self.config)?;
        do_handshake(&mut transport).await?;
        authenticate(&self.config, &token, &mut transport).await?;

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

        let _ = tokio::join!(
            console_task.spawn(Shutdown::new(self._notify_shutdown.subscribe())),
            receive_task.spawn(Shutdown::new(self._notify_shutdown.subscribe())),
            forward_task.spawn(Shutdown::new(self._notify_shutdown.subscribe())),
            ping_task.spawn(Shutdown::new(self._notify_shutdown.subscribe()))
        );

        Ok(())
    }
}

fn get_token(config: &Arc<Config>) -> Result<String> {
    let auth_manager = match config.lock_auth_manager() {
        Ok(lock) => lock,
        Err(err) => {
            error!("error when trying to lock context_manager: {}", err);
            return Err(err.into());
        }
    };

    let token = auth_manager
        .current_token()
        .clone()
        .unwrap_or(String::default());

    Ok(token.to_owned())
}

async fn get_transport(app_context: &AppContext) -> Result<TcpFrameTransport> {
    info!("Trying to connect...");

    let addr = ServerAddr::try_from(app_context.clone())
        .unwrap()
        .to_socket_addr()?;
    let transport = TcpFrameTransport::connect(addr).await?;

    Ok(transport)
}

fn get_context(args: &Arc<ListenArgs>, config: &Arc<Config>) -> Result<AppContext> {
    let contexts = match config.lock_context_manager() {
        Ok(lock) => lock,
        Err(err) => {
            error!("error when trying to lock context_manager: {}", err);
            return Err(err.into());
        }
    };
    let fallback = contexts.default_context_str().to_string();
    let context_name = args.app_context().unwrap_or(fallback);

    match contexts.get_context(&context_name) {
        Some(ctx) => Ok(ctx),
        None => Err(format!("context {} was not found.", context_name).into()),
    }
}

async fn authenticate(
    config: &Arc<Config>,
    token: &str,
    client: &mut TcpFrameTransport,
) -> Result<()> {
    let grant_type = GrantType::TOKEN(TokenAuthenticationArgs::new(token));
    let authenticate_frame = TcpFrame::Authenticate(Authenticate::new(grant_type));

    match client.send_frame(&authenticate_frame).await? {
        TcpFrame::AuthenticateAck(data) => {
            debug!("authenticated successfully");
            if String::default() == data.token() {
                return Ok(());
            }

            debug!("trying to save user token into config file..");
            let mut auth_manager = config.lock_auth_manager()?;

            // Stores user token into local config file
            let token = AuthToken::from(data.token());
            auth_manager.set_current_token(Some(token));

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
    info!("Connected to server, trying handshake...");

    let frame = TcpFrame::ClientConnected(ClientConnected);
    match client.send_frame(&frame).await? {
        TcpFrame::ClientConnectedAck(_) => Ok(()),
        actual => {
            debug!("received invalid frame when doing handshake. received {} instead of ClientConnectedAck", actual);
            Err("failed to do handshake with server.".into())
        }
    }
}
