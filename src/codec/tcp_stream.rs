use std::fmt::Display;

use bytes::BytesMut;
use uuid::Uuid;

pub enum TcpFrame {
    ClientConnected,
    Ping,
    Pong,
    ClientConnectedAck {
        port: u16,
    },
    RemoteSocketDisconnected {
        connection_id: Uuid
    },
    IncomingSocket {
        connection_id: Uuid
    },
    ClientUnableToConnect {
        connection_id: Uuid
    },
    LocalClientDisconnected {
        connection_id: Uuid
    },
    DataPacket {
        connection_id: Uuid,
        buffer: BytesMut,
    },
}

impl Display for TcpFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data_type = match self {
            TcpFrame::ClientConnected => "ClientConnected".to_string(),
            TcpFrame::ClientConnectedAck { port } => format!("ClientConnectedACK ({})", port),
            TcpFrame::ClientUnableToConnect { connection_id } => format!("ClientUnableToConnect ({})", connection_id),
            TcpFrame::DataPacket { connection_id, buffer } => format!("DataPacket, {}, size: {}", connection_id, buffer.len()),
            TcpFrame::LocalClientDisconnected { connection_id } => format!("LocalClientDisconnected ({})", connection_id),
            TcpFrame::Ping => format!("Ping"),
            TcpFrame::Pong => format!("Pong"),
            _ => "Invalid Tcp Frame".to_string(),
        };

        let msg = format!("Received new tcpframe: {}", data_type);
        write!(f, "{}", msg)
    }
}