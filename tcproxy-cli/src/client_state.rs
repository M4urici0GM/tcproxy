use bytes::BytesMut;
use std::collections::HashMap;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::debug;
use uuid::Uuid;

pub struct ClientState {
    connections: Mutex<HashMap<Uuid, (Sender<BytesMut>, CancellationToken)>>,
}

impl ClientState {
    pub fn new() -> Self {
        Self {
            connections: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert_connection(
        &self,
        connection_id: Uuid,
        sender: Sender<BytesMut>,
        cancellation_token: CancellationToken,
    ) {
        let mut lock = self.connections.lock().unwrap();
        lock.insert(connection_id, (sender, cancellation_token));
    }

    pub fn get_connection(&self, id: Uuid) -> Option<(Sender<BytesMut>, CancellationToken)> {
        let lock = self.connections.lock().unwrap();
        match lock.get(&id) {
            Some((sender, token)) => {
                debug!("connection {} not found", id);
                Some((sender.clone(), token.clone()))
            }
            None => None,
        }
    }

    pub fn remove_connection(&self, id: Uuid) -> Option<(Sender<BytesMut>, CancellationToken)> {
        let mut lock = self.connections.lock().unwrap();
        if !lock.contains_key(&id) {
            return None;
        }

        Some(lock.remove(&id).unwrap())
    }
}
