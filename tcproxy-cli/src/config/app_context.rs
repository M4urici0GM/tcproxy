use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};
use serde::{Deserialize, Serialize};
use tcproxy_core::Error;

#[derive(Debug)]
pub enum AppContextError {
    DoesntExist(String),
    AlreadyExists(AppContext),
    Other(Error),
}

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

impl std::error::Error for AppContextError {}

impl Display for AppContextError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            AppContextError::DoesntExist(ctx_name) => format!("context {} doesn't exists.", ctx_name),
            AppContextError::AlreadyExists(ctx) => format!("context {} with ip {} already exists", ctx.name(), ctx.ip()),
            AppContextError::Other(err) => format!("unexpected error: {}", err),
        };

        write!(f, "{}", msg)
    }
}

impl From<String> for AppContextError {
    fn from(msg: String) -> Self {
        AppContextError::Other(msg.into())
    }
}

impl From<&str> for AppContextError {
    fn from(msg: &str) -> Self {
        AppContextError::Other(msg.into())
    }
}