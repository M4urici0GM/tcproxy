use std::{collections::HashMap, sync::Mutex};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::trace;

type ConnectionCollection = HashMap<u32, (Sender<Vec<u8>>, CancellationToken)>;

#[derive(Debug)]
pub struct ConnectionsManager {
    last_connection_id: Mutex<u32>,
    connections: Mutex<ConnectionCollection>,
}

impl Default for ConnectionsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionsManager {
    pub fn new() -> Self {
        ConnectionsManager {
            last_connection_id: Mutex::new(0),
            connections: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert_connection(
        &self,
        sender: Sender<Vec<u8>>,
        cancellation_token: CancellationToken,
    ) -> u32 {
        let mut last_id = self.last_connection_id.lock().unwrap();
        let mut state = self.connections.lock().unwrap();

        let new_id = *last_id + 1u32;
        *last_id = new_id;

        state.insert(new_id, (sender, cancellation_token));

        new_id
    }

    pub fn remove_connection(
        &self,
        connection_id: &u32,
    ) -> Option<(Sender<Vec<u8>>, CancellationToken)> {
        let mut state = self.connections.lock().unwrap();
        if !state.contains_key(connection_id) {
            return None;
        }

        Some(state.remove(connection_id).unwrap())
    }

    pub fn get_connection(
        &self,
        connection_id: &u32,
    ) -> Option<(Sender<Vec<u8>>, CancellationToken)> {
        let state = self.connections.lock().unwrap();
        match state.get(connection_id) {
            Some(item) => Some(item.clone()),
            None => {
                trace!("connection {} not found in state", connection_id);
                None
            }
        }
    }
}
