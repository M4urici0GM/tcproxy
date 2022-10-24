use std::ops::Range;
use std::sync::Arc;
use tcproxy_core::AsyncCommand;

use crate::managers::{ConnectionsManager, PortManager};

#[derive(Debug)]
pub struct ClientState {
    pub(crate) connections: Arc<ConnectionsManager>,
    pub(crate) ports: Arc<PortManager>,
}

impl ClientState {
    pub fn new(port_range: Range<u16>) -> Arc<Self> {
        Arc::new(Self {
            connections: Arc::new(ConnectionsManager::new()),
            ports: Arc::new(PortManager::new(port_range)),
        })
    }
}
