use std::net::{IpAddr, SocketAddr};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppContext {
    name: String,
    target_ip: IpAddr,
    target_port: u16,
}

impl AppContext {
    pub fn new(name: &str, ip: &SocketAddr) -> Self {
        Self {
            name: String::from(name),
            target_ip: ip.ip(),
            target_port: ip.port()
        }
    }

    pub fn port(&self) -> &u16 {
        &self.target_port
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ip(&self) -> &IpAddr {
        &self.target_ip
    }
}
