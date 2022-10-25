use std::fmt::{Display, Formatter};
use rand::Rng;
use std::ops::Range;
use std::sync::Arc;
use std::sync::Mutex;
use tcproxy_core::{Error};
use tracing::{debug, error};

#[derive(Debug, Clone)]
pub struct PortManager {
    initial_port: u16,
    final_port: u16,
    used_ports: Arc<Mutex<Vec<u16>>>,
}

#[derive(Debug)]
pub enum PortError {
    PortLimitReached(Error),
    Other(Error),
}

impl PortManager {
    pub fn new(port_range: Range<u16>) -> Self {
        Self {
            initial_port: port_range.start,
            final_port: port_range.end,
            used_ports: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn remove_port(&self, target_port: u16) {
        let mut mutex_lock = self.used_ports.lock().unwrap();
        mutex_lock.retain(|port| *port != target_port);

        debug!("removed port {} from available proxies.", target_port);
    }

    pub async fn get_port(&self) -> std::result::Result<u16, PortError> {
        let mut mutex_lock = self.used_ports.lock().unwrap();

        let mut rng = rand::thread_rng();
        let mut random_port = rng.gen_range(self.initial_port..self.final_port);
        let mut tries = 0;
        while mutex_lock.contains(&random_port) {
            tries += 1;
            random_port = rng.gen_range(self.initial_port..self.final_port);

            if tries == mutex_lock.len() {
                error!("could not accept more connections, all ports used.");
                return Err(PortError::PortLimitReached("Port limit reached.".into()));
            }
        }

        mutex_lock.push(random_port);
        Ok(random_port)
    }
}


impl Display for PortError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            PortError::PortLimitReached(_) => "PortLimit has reached".to_owned(),
            err => format!("unknow error: {}", err),
        };

        write!(f, "{}", msg)
    }
}

impl std::error::Error for PortError {

}