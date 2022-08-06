use std::net::Ipv4Addr;
use std::sync::Arc;
use tcproxy_core::TcpFrame;
use tokio::sync::mpsc::Sender;

use crate::ProxyState;

pub struct ProxyServer {
    pub(crate) listen_ip: Ipv4Addr,
    pub(crate) port: u16,
    pub(crate) host_sender: Sender<TcpFrame>,
    pub(crate) available_connections: Arc<ProxyState>,
}

