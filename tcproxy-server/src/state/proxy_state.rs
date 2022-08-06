use std::sync::Arc;
use std::ops::Range;

use crate::managers::{ConnectionsManager, PortManager};

#[derive(Debug)]
pub struct ProxyState {
    pub(crate) connections: Arc<ConnectionsManager>,
    pub(crate) ports: Arc<PortManager>,
}

impl ProxyState {
    pub fn new(port_range: &Range<u16>) -> Self {
        Self {
            connections: Arc::new(ConnectionsManager::new()),
            ports: Arc::new(PortManager::new(port_range.clone())),
        }
    }
}