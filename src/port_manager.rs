use std::sync::Arc;
use std::sync::Mutex;
use rand::Rng;
use tracing::error;

use crate::Result;

#[derive(Debug, Clone)]
pub struct PortManager {
    pub(crate) initial_port: u16,
    pub(crate) final_port:  u16,
    pub(crate) available_proxies: Arc<Mutex<Vec<u16>>>
}
impl PortManager {
    pub async fn get_port(&self) -> Result<u16> {
        let mut mutex_lock = self.available_proxies.lock().unwrap();

        let mut rng = rand::thread_rng();
        let mut random_port = rng.gen_range(self.initial_port..self.final_port);
        let mut tries = 0;
        while mutex_lock.contains(&random_port) {
            tries += 1;
            random_port = rng.gen_range(self.initial_port..self.final_port);

            if tries == mutex_lock.len() {
                error!("could not accept more connections, all ports used.");
                return Err("Port limit reached.".into());
            }
        }

        mutex_lock.push(random_port);
        Ok(random_port)
    }
}
