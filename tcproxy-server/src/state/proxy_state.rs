use std::sync::Arc;

use crate::managers::{
    AuthenticationManagerGuard, ConnectionsManager, PortManagerGuard, UserManager,
};
use crate::ServerConfig;

pub struct ClientState {
    is_authenticated: bool,
    server_config: Arc<ServerConfig>,
    port_manager: Arc<PortManagerGuard>,
    auth_manager: Arc<AuthenticationManagerGuard>,
    accounts_manager: Arc<Box<dyn UserManager + 'static>>,
    connection_manager: Arc<ConnectionsManager>,
}

impl ClientState {
    pub fn new(
        port_manager: Arc<PortManagerGuard>,
        auth_manager: Arc<AuthenticationManagerGuard>,
        server_config: &Arc<ServerConfig>,
        account_manager: &Arc<Box<dyn UserManager + 'static>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            auth_manager,
            port_manager,
            is_authenticated: false,
            server_config: server_config.clone(),
            accounts_manager: account_manager.clone(),
            connection_manager: Arc::new(ConnectionsManager::new()),
        })
    }

    pub fn get_port_manager(&self) -> &Arc<PortManagerGuard> {
        &self.port_manager
    }

    pub fn get_connection_manager(&self) -> &Arc<ConnectionsManager> {
        &self.connection_manager
    }

    pub fn get_accounts_manager(&self) -> &Arc<Box<dyn UserManager + 'static>> {
        &self.accounts_manager
    }

    pub fn get_server_config(&self) -> &Arc<ServerConfig> {
        &self.server_config
    }

    pub fn get_auth_manager(&self) -> &Arc<AuthenticationManagerGuard> {
        &self.auth_manager
    }
}
