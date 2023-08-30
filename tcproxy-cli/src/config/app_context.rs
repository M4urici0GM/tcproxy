use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::server_addr::ServerAddr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppContext {
    name: String,
    target_host: String,
    target_port: u16,
    tls: bool
}

impl AppContext {
    pub fn new(name: &str, host: &str, port: &u16, use_ssl: bool) -> Self {
        Self {
            name: String::from(name),
            target_host: host.to_owned(),
            target_port: port.to_owned(),
            tls: use_ssl 
        }
    }

    pub fn from_addr(name: &str, addr: &ServerAddr, use_ssl: bool) -> Self {
        Self::new(name, addr.host(), addr.port(), use_ssl)
    }

    pub fn host(&self) -> &str {
        &self.target_host
    }

    pub fn port(&self) -> &u16 {
        &self.target_port
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tls(&self) -> bool {
        self.tls
    }
}

impl From<ServerAddr> for AppContext {
    fn from(_value: ServerAddr) -> Self {
        todo!()
    }
}

impl Display for AppContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[name = {}, host = {}, port = {}]",
            self.name, self.target_host, self.target_host
        )
    }
}
