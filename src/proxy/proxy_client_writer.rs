use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use tracing::{error,  debug};

use crate::Result;
use crate::codec::TcpFrame;

pub struct ProxyClientStreamWriter {
    pub(crate) proxy_client_receiver: Receiver<TcpFrame>,
    pub(crate) writer: OwnedWriteHalf,
    pub(crate) cancellation_token: CancellationToken,
}

impl ProxyClientStreamWriter {
    async fn receive_frame(&mut self) -> Result<()> {
        loop {
            let msg = match self.proxy_client_receiver.recv().await {
                Some(msg) => msg,
                None => break,
            };

            let mut buff = msg.to_buffer();
            match self.writer.write_buf(&mut buff).await {
                Ok(s) => {
                    debug!("Send data packet to client.. {}", msg);
                    debug!("written {} bytes to client.. ", s);
                }
                Err(err) => {
                    error!("Failed to send packet to client.. {}", err);
                    return Err(err.into());
                }
            };
        }

        Ok(())
    }

    pub async fn write_to_socket(&mut self) -> Result<()> {
        let token = self.cancellation_token.child_token();

        tokio::select! {
            _ = self.receive_frame() => {},
            _ = token.cancelled() => {},
        };

        Ok(())
    }
}