use std::sync::Arc;

use crate::ServerConfig;

pub trait FeatureManager: Sync + Send {
    fn get_config(&self) -> Arc<ServerConfig>;
}

#[derive(Debug)]
pub struct DefaultFeatureManager {
    server_config: Arc<ServerConfig>
}

impl DefaultFeatureManager {
    pub fn new(server_config: ServerConfig) -> Self {
        Self {
            server_config: Arc::new(server_config),
        }
    }
}

impl FeatureManager for DefaultFeatureManager {
    fn get_config(&self) -> Arc<ServerConfig> {
        self.server_config.clone()
    }
}