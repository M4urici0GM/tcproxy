use bytes::BytesMut;
use uuid::Uuid;

pub enum TcpFrame {
    ClientConnected,
    IncomingSocket {
        connection_id: Uuid
    },
    DataPacket {
        connection_id: Uuid,
        buffer: BytesMut,
    },
}

