use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc};
use bytes::BytesMut;
use uuid::Uuid;
use tracing::{debug, error};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::Result;
use crate::codec::TcpFrame;

pub struct TcpConnection {
    pub(crate) connection_id: Uuid,
    pub(crate) _connection_addr: SocketAddr,
    pub(crate) host_sender: Sender<TcpFrame>,
    pub(crate) available_connections: Arc<Mutex<HashMap<Uuid, Sender<BytesMut>>>>,
}
