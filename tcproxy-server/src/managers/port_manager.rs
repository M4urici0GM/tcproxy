use rand::Rng;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use tcproxy_core::Error;
use tracing::log::{debug, error, warn};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PortPermit {
    connection_id: u32,
    connection_token: String,
    used_port: u16,
}

impl PortPermit {
    pub fn new(conn_id: &u32, token: &str, port: &u16) -> Self {
        Self {
            connection_id: *conn_id,
            connection_token: String::from(token),
            used_port: *port,
        }
    }

    pub fn port(&self) -> &u16 {
        &self.used_port
    }

    pub fn connection_id(&self) -> &u32 {
        &self.connection_id
    }

    pub fn connection_token(&self) -> &str {
        &self.connection_token
    }
}

pub struct PortManager(Arc<Mutex<NetworkPortPool>>);

impl From<NetworkPortPool> for PortManager {
    fn from(value: NetworkPortPool) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }
}

impl PortManager {
    pub fn free_port(&self, permit: PortPermit) {
        let mut lock = self.0.lock().unwrap();

        debug!("disposing used port: {permit}");
        lock.free_port(permit);
    }

    pub fn reserve_port(&self, conn_id: &u32, conn_token: &str) -> Result<PortPermit, PortError> {
        let mut lock = self.0.lock().unwrap();

        match lock.reserve_port(conn_id, conn_token) {
            Err(PortError::PortLimitReached) => {
                warn!("port limit reached!.");
                Err(PortError::PortLimitReached)
            }
            Err(err) => {
                error!("failed when trying to reserve port: {err}");
                Err(err)
            }
            actual => actual,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkPortPool {
    used_ports: HashSet<PortPermit>,
    available_ports: Vec<u16>,
}

#[derive(Debug)]
pub enum PortError {
    PortLimitReached,
    Other(Error),
}

impl NetworkPortPool {
    pub fn new(port_range: Range<u16>) -> Self {
        let mut available_ports = Vec::new();
        for i in port_range.start..port_range.end {
            available_ports.push(i);
        }

        Self {
            used_ports: HashSet::new(),
            available_ports,
        }
    }

    pub fn used_ports(&self) -> &HashSet<PortPermit> {
        &self.used_ports
    }

    pub fn available_ports(&self) -> &[u16] {
        &self.available_ports
    }

    pub fn free_port(&mut self, permit: PortPermit) {
        if !self.used_ports.contains(&permit) {
            warn!("no port permit found with: {}", &permit);
            return;
        }

        self.used_ports.remove(&permit);
        self.available_ports.push(*permit.port());
    }

    pub fn reserve_port(
        &mut self,
        conn_id: &u32,
        conn_token: &str,
    ) -> Result<PortPermit, PortError> {
        if self.available_ports.is_empty() {
            return Err(PortError::PortLimitReached);
        }

        let mut rng = rand::thread_rng();
        let random_idx = rng.gen_range(0..self.available_ports.len());
        let selected_port = self.available_ports.remove(random_idx);

        let port_permit = PortPermit::new(conn_id, conn_token, &selected_port);
        self.used_ports.insert(port_permit.clone());

        Ok(port_permit)
    }
}

impl Display for PortError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            PortError::PortLimitReached => "PortLimit has reached".to_owned(),
            err => format!("unknow error: {}", err),
        };

        write!(f, "{}", msg)
    }
}

impl Display for PortPermit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = format!(
            "PortPermit[token = {}, port = {}, conn_id = {}]",
            self.connection_token, self.used_port, self.connection_id
        );

        write!(f, "{}", msg)
    }
}
impl std::error::Error for PortError {}

#[cfg(test)]
pub mod tests {
    use super::NetworkPortPool;

    #[test]
    pub fn should_be_able_to_reserve_port() {
        // Arrange
        let conn_id = 2u32;
        let connection_token = "some_token";
        let mut port_manager = NetworkPortPool::new(10..20);

        // Act
        let port_permit = port_manager
            .reserve_port(&conn_id, connection_token)
            .unwrap();

        // Assert
        assert!(!port_manager.available_ports.contains(port_permit.port()));
        assert_eq!(&conn_id, port_permit.connection_id());
        assert_eq!(connection_token, port_permit.connection_token());
    }

    #[test]
    pub fn reserved_port_should_be_within_range() {
        // Arrange
        let min_port = 10;
        let max_port = 20;
        let conn_id = 2u32;
        let connection_token = "some_token";
        let mut port_manager = NetworkPortPool::new(min_port..max_port);

        // Act
        let port_permit = port_manager
            .reserve_port(&conn_id, connection_token)
            .unwrap();

        // Assert
        assert!(port_permit.port() >= &min_port);
        assert!(port_permit.port() <= &max_port);
    }

    #[test]
    pub fn should_be_able_to_free_port() {
        // Arrange
        let conn_id = 2u32;
        let connection_token = "some_token";
        let mut port_manager = NetworkPortPool::new(10..20);

        // Act
        let port_permit = port_manager
            .reserve_port(&conn_id, connection_token)
            .unwrap();

        port_manager.free_port(port_permit.clone());

        // Assert
        assert_eq!(0, port_manager.used_ports().len());
        assert!(port_manager.available_ports.contains(port_permit.port()));
    }
}
