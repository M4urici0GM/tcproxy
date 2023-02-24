use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use mongodb::{Client, Database};
use tcproxy_core::Result;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use tcproxy_core::tcp::{ISocketListener, SocketConnection, SocketListener};
use crate::managers::{AccountManager, AuthenticationManager, AuthenticationManagerGuard, DefaultAccountManager, FeatureManager, IFeatureManager, PortManager, PortManagerGuard};

use crate::proxy::ClientConnection;

/// Represents the server application
pub struct Server {
    feature_manager: Arc<IFeatureManager>,
    server_listener: ISocketListener,
    db_client: Arc<Client>,
    database: Arc<Database>,
}

impl Server {
    pub fn new<TListener, TFeatureManager>(feature_manager: TFeatureManager, listener: TListener, db_client: Client) -> Self
    where
        TListener: SocketListener + 'static,
        TFeatureManager: FeatureManager + 'static
    {
        let db = db_client.database("tcproxy");
        info!("using {} as default database", db.name());
        Self {
            feature_manager: Arc::new(Box::new(feature_manager)),
            server_listener: Box::new(listener),
            db_client: Arc::new(db_client),
            database: Arc::new(db),
        }
    }

    pub async fn run(&mut self, shutdown_signal: impl Future) -> Result<()> {
        let cancellation_token = CancellationToken::new();
        tokio::select! {
            _ = self.start(cancellation_token.child_token()) => {},
            _ = shutdown_signal => {
                info!("server is being shut down.");
                cancellation_token.cancel();
            }
        };

        Ok(())
    }

    pub fn get_listen_ip(&self) -> Result<SocketAddr> {
        self.server_listener.listen_ip()
    }

    async fn start(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        info!("server running at: {}", self.feature_manager.get_config().get_listen_port());
        loop {
            let socket = self.server_listener.accept().await?;
            info!("received new socket from {}", socket.addr);

            let cancellation_token = cancellation_token.child_token();
            self.spawn_proxy_connection(socket, cancellation_token);
        }
    }

    fn spawn_proxy_connection<T>(
        &self,
        socket: T,
        cancellation_token: CancellationToken,
    ) -> JoinHandle<Result<()>>
    where
        T: SocketConnection + 'static,
    {
        let server_config = self.feature_manager.get_config();
        let auth_manager = AuthenticationManager::new();
        let port_manager = PortManager::new(server_config.get_port_range());
        let account_manager: Arc<Box<dyn AccountManager + 'static>> = Arc::new(Box::new(DefaultAccountManager::new(&self.database)));

        let port_guard = Arc::new(PortManagerGuard::new(port_manager));
        let auth_guard = Arc::new(AuthenticationManagerGuard::new(auth_manager));

        let mut proxy_client = ClientConnection::new(
            port_guard,
            auth_guard,
            &server_config,
            &account_manager);

        tokio::spawn(async move {
            let socket_addr = socket.addr();

            match proxy_client
                .start_streaming(socket, cancellation_token)
                .await
            {
                Ok(_) => debug!("Socket {} has been closed gracefully.", socket_addr),
                Err(err) => debug!(
                    "Socket {} has been disconnected with error.. {}",
                    socket_addr, err
                ),
            };

            Ok(())
        })
    }
}
