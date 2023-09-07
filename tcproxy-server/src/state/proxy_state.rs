use std::sync::Arc;

use crate::managers::{
    AuthenticationManagerGuard, ConnectionsManager, NetworkPortPool, PortManager, UserManager,
};
use crate::ServerConfig;

pub struct ClientState {
    server_config: Arc<ServerConfig>,
    port_manager: PortManager,
    auth_manager: Arc<AuthenticationManagerGuard>,
    accounts_manager: Arc<dyn UserManager + 'static>,
    connection_manager: Arc<ConnectionsManager>,
}

impl ClientState {
    pub fn new(
        port_manager: PortManager,
        auth_manager: Arc<AuthenticationManagerGuard>,
        server_config: &Arc<ServerConfig>,
        account_manager: &Arc<impl UserManager + 'static>,
    ) -> Arc<Self> {
        Arc::new(Self {
            auth_manager,
            port_manager,
            server_config: server_config.clone(),
            accounts_manager: account_manager.clone(),
            connection_manager: Arc::new(ConnectionsManager::new()),
        })
    }

    pub fn get_port_manager(&self) -> &PortManager {
        &self.port_manager
    }

    pub fn get_connection_manager(&self) -> &Arc<ConnectionsManager> {
        &self.connection_manager
    }

    pub fn get_accounts_manager(&self) -> &Arc<dyn UserManager + 'static> {
        &self.accounts_manager
    }

    pub fn get_server_config(&self) -> &Arc<ServerConfig> {
        &self.server_config
    }

    pub fn get_auth_manager(&self) -> &Arc<AuthenticationManagerGuard> {
        &self.auth_manager
    }
}
