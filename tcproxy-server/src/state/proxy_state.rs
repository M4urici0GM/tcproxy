use std::sync::Arc;

use crate::managers::{ConnectionsManager, IFeatureManager, PortManager, PortManagerGuard};

pub struct ClientState {
    port_manager: Arc<PortManagerGuard>,
    connection_manager: Arc<ConnectionsManager>,
}

impl ClientState {
    pub fn new(feature_manager: &Arc<IFeatureManager>) -> Arc<Self> {
        let server_config = feature_manager.get_config();
        let port_manager = PortManager::new(server_config.get_port_range());

        Arc::new(Self {
            connection_manager: Arc::new(ConnectionsManager::new()),
            port_manager: Arc::new(PortManagerGuard::new(port_manager)),
        })
    }

    pub fn get_port_manager(&self) -> &Arc<PortManagerGuard> {
        &self.port_manager
    }

    pub fn get_connection_manager(&self) -> &Arc<ConnectionsManager> {
        &self.connection_manager
    }
}
