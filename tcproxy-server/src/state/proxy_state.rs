use std::sync::Arc;

use crate::managers::{ConnectionsManager, IFeatureManager, PortManager};

pub struct ClientState {
    port_manager: Arc<PortManager>,
    feature_manager: Arc<IFeatureManager>,
    connection_manager: Arc<ConnectionsManager>,
}

impl ClientState {
    pub fn new(feature_manager: &Arc<IFeatureManager>) -> Arc<Self> {
        let server_config = feature_manager.get_config();
        Arc::new(Self {
            connection_manager: Arc::new(ConnectionsManager::new()),
            port_manager: Arc::new(PortManager::new(server_config.get_port_range())),
            feature_manager: feature_manager.clone(),
        })
    }

    pub fn get_port_manager(&self) -> Arc<PortManager> {
        self.port_manager.clone()
    }

    pub fn get_connection_manager(&self) -> Arc<ConnectionsManager> {
        self.connection_manager.clone()
    }
}
