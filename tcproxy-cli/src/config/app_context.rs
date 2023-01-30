use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppContext {
    name: String,
    target_host: String,
    target_port: u16,
}

impl AppContext {
    pub fn new(name: &str, host: &str, port: &u16) -> Self {
        Self {
            name: String::from(name),
            target_host: host.to_owned(),
            target_port: port.to_owned()
        }
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
}

impl Display for AppContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[name = {}, host = {}, port = {}]", self.name, self.target_host, self.target_host)
    }
}
