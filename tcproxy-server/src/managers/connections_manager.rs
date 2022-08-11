use bytes::BytesMut;
use std::{collections::HashMap, sync::Mutex};
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::trace;
use uuid::Uuid;

type ConnectionCollection = HashMap<Uuid, (Sender<BytesMut>, CancellationToken)>;

#[derive(Debug)]
pub struct ConnectionsManager {
    connections: Mutex<ConnectionCollection>,
}

impl ConnectionsManager {
    pub fn new() -> Self {
        ConnectionsManager {
            connections: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert_connection(
        &self,
        connection_id: Uuid,
        sender: Sender<BytesMut>,
        cancellation_token: CancellationToken,
    ) {
        let mut state = self.connections.lock().unwrap();
        state.insert(connection_id, (sender, cancellation_token));
    }

    pub fn remove_connection(
        &self,
        connection_id: Uuid,
    ) -> Option<(Sender<BytesMut>, CancellationToken)> {
        let mut state = self.connections.lock().unwrap();
        if !state.contains_key(&connection_id) {
            return None;
        }

        Some(state.remove(&connection_id).unwrap())
    }

    pub fn get_connection(
        &self,
        connection_id: Uuid,
    ) -> Option<(Sender<BytesMut>, CancellationToken)> {
        let state = self.connections.lock().unwrap();
        match state.get(&connection_id) {
            Some(item) => Some(item.clone()),
            None => {
                trace!("connection {} not found in state", connection_id);
                return None;
            }
        }
    }
}
