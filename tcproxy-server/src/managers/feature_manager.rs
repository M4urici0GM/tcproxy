use std::sync::Arc;
use crate::ServerConfig;

pub trait FeatureManager {
    fn get_config(&self) -> Arc<ServerConfig>;
}

#[derive(Debug)]
pub struct DefaultFeatureManager {
    server_config: Arc<ServerConfig>
}

impl FeatureManager for DefaultFeatureManager {
    fn get_config(&self) -> Arc<ServerConfig> {
        self.server_config.clone()
    }
}