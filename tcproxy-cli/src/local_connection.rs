use bytes::BytesMut;
use std::net::SocketAddrV4;
use tcproxy_core::{ClientPacketData, Result, TcpFrame};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::debug;
use uuid::Uuid;
use tcproxy_core::TcpFrame::ClientPacket;

pub struct LocalConnection {
    connection_id: Uuid,
    target_ip: SocketAddrV4,
    sender: Sender<TcpFrame>,
}

impl LocalConnection {
    pub fn new(connection_id: Uuid, sender: &Sender<TcpFrame>, target_ip: SocketAddrV4) -> Self {
        Self {
            target_ip,
            connection_id,
            sender: sender.clone(),
        }
    }

    async fn connect(&self) -> Result<TcpStream> {
        match TcpStream::connect(self.target_ip).await {
            Ok(stream) => Ok(stream),
            Err(err) => {
                debug!(
                    "Error when connecting to {}: {}. Aborting connection..",
                    "192.168.0.221:22", err
                );

                let _ = self
                    .sender
                    .send(TcpFrame::ClientUnableToConnect { connection_id: self.connection_id })
                    .await;

                return Err(err.into());
            }
        }
    }

    fn read_from_socket(
        mut reader: OwnedReadHalf,
        sender: Sender<TcpFrame>,
        connection_id: Uuid,
    ) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let mut buffer = BytesMut::with_capacity(1024 * 8);
            loop {
                let bytes_read = reader.read_buf(&mut buffer).await?;
                if 0 == bytes_read {
                    debug!("reached end of stream");
                    return Ok(());
                }

                let tcp_frame = TcpFrame::ClientPacket(ClientPacketData::new(
                    connection_id,
                    buffer.split_to(bytes_read),
                    bytes_read as u32,
                ));

                sender.send(tcp_frame).await?;
            }
        })
    }

    fn write_to_socket(
        mut writer: OwnedWriteHalf,
        mut reader: Receiver<BytesMut>,
    ) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            loop {
                let result = reader.recv().await;
                if result == None {
                    break;
                }

                let mut msg = result.unwrap();
                let bytes_written = writer.write_buf(&mut msg).await?;
                writer.flush().await?;

                debug!("written {} bytes to target stream", bytes_written);
            }

            reader.close();
            Ok(())
        })
    }

    pub async fn read_from_local_connection(
        &mut self,
        reader: Receiver<BytesMut>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        let connection = self.connect().await?;
        let (stream_reader, stream_writer) = connection.into_split();
        let task1 = LocalConnection::read_from_socket(
            stream_reader,
            self.sender.clone(),
            self.connection_id.clone(),
        );

        let task2 = LocalConnection::write_to_socket(stream_writer, reader);

        tokio::select! {
            _ = task2 => {},
            _ = task1 => {},
            _ = cancellation_token.cancelled() => {}
        };

        if cancellation_token.is_cancelled() {
            return Ok(());
        }

        debug!("connection {} disconnected!", self.connection_id);
        Ok(())
    }
}
