use bytes::BytesMut;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use tracing::debug;

pub struct ClientState {
    console_sender: Sender<i32>,
    remote_ip: Mutex<String>,
    last_sent_ping: Mutex<u32>,
    last_ping: Mutex<u32>,
    connections: Mutex<HashMap<u32, (Sender<BytesMut>, CancellationToken)>>,
}

pub struct ConsoleStatus {
    pub remote_ip: SocketAddr,
    pub ping: f64,
    pub connections: i32,
}


impl ClientState {
    pub fn new(console_sender: &Sender<i32>) -> Self {
        Self {
            remote_ip: Mutex::new(String::from("")),
            connections: Mutex::new(HashMap::new()),
            last_sent_ping: Mutex::new(0),
            last_ping: Mutex::new(0),
            console_sender: console_sender.clone(),
        }
    }

    pub fn update_last_sent_ping(&self, time: DateTime<Utc>) {
        let mut last_sent_ping = self.last_sent_ping.lock().unwrap();
        *last_sent_ping = time.timestamp_subsec_millis();
    }

    pub fn update_last_ping(&self, time: DateTime<Utc>) {
        let mutex_lock = self.last_sent_ping.lock().unwrap();
        let mut last_ping = self.last_ping.lock().unwrap();

        *last_ping = time.timestamp_subsec_millis() - *mutex_lock;
        drop(mutex_lock);

        self.notify_console_update();
    }

    pub fn update_remote_ip(&self, ip: &str) {
        let mut mutex = self.remote_ip.lock().unwrap();
        *mutex = String::from(ip);

        self.notify_console_update();
    }

    pub fn get_console_status(&self) -> ConsoleStatus {
        let remote_ip = self.remote_ip.lock().unwrap();
        let ping = *self.last_ping.lock().unwrap() as f64;
        let connections = self.connections.lock().unwrap();

        let remote_ip_str = remote_ip.clone();
        let connections_len = connections.len();

        let remote_ip = match remote_ip_str.as_str() {
            "" => SocketAddr::new(
                std::net::IpAddr::V4(Ipv4Addr::from_str("0.0.0.0").unwrap()),
                8080u16,
            ),
            value => SocketAddr::new(
                std::net::IpAddr::V4(Ipv4Addr::from_str(value).unwrap()),
                value.parse::<u16>().unwrap(),
            ),
        };

        ConsoleStatus {
            ping,
            remote_ip,
            connections: connections_len as i32,
        }
    }

    pub fn insert_connection(
        &self,
        connection_id: &u32,
        sender: Sender<BytesMut>,
        cancellation_token: CancellationToken,
    ) {
        let mut lock = self.connections.lock().unwrap();
        lock.insert(*connection_id, (sender, cancellation_token));
        drop(lock);

        self.notify_console_update();
    }

    pub fn get_connection(&self, id: &u32) -> Option<(Sender<BytesMut>, CancellationToken)> {
        let lock = self.connections.lock().unwrap();
        match lock.get(id) {
            Some((sender, token)) => Some((sender.clone(), token.clone())),
            None => {
                debug!("connection {} not found", id);
                None
            }
        }
    }

    pub fn remove_connection(&self, id: &u32) -> Option<(Sender<BytesMut>, CancellationToken)> {
        debug!("removing connection {}", id);
        let mut lock = self.connections.lock().unwrap();
        if !lock.contains_key(id) {
            return None;
        }

        let result = lock.remove(id).unwrap();

        self.notify_console_update();
        Some(result)
    }

    fn notify_console_update(&self) {
        let sender = self.console_sender.clone();
        tokio::spawn(async move {
            let _ = sender.send(0).await;
            drop(sender);
        });
    }
}
