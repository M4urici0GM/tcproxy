use std::net::SocketAddr;
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, debug};
use uuid::Uuid;

use crate::Result;
use crate::codec::{TcpFrame, TcpFrameCodec};

pub struct ProxyClientStreamWriter {
    pub(crate) remote_ip: SocketAddr,
    pub(crate) proxy_client_receiver: Receiver<TcpFrame>,
    pub(crate) writer: SplitSink<Framed<TcpStream, TcpFrameCodec>, TcpFrame>,
}

impl ProxyClientStreamWriter {
    pub async fn write_to_socket(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        while !cancellation_token.is_cancelled() {
            let msg = match self.proxy_client_receiver.recv().await {
                Some(msg) => msg,
                None => break,
            };

            match self.writer.send(msg).await {
                Ok(_) => {
                    debug!("Send data packet to client..");
                }
                Err(err) => {
                    error!("Failed to send packet to client.. {}", err);
                    return Err(err.into());
                }
            };
        }

        Ok(())
    }
}