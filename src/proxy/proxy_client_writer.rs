use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::Result;
use crate::codec::{TcpFrame, TcpFrameCodec};

pub struct ProxyClientStreamWriter {
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

            if let TcpFrame::DataPacket { connection_id, buffer } = msg {
                let tcp_frame = TcpFrame::DataPacket { connection_id, buffer };
                match self.writer.send(tcp_frame).await {
                    Ok(_) => {
                        info!("send new packet to {}", connection_id);
                    }
                    Err(err) => {
                        error!("error sending packet to {}: {}", connection_id, err);
                        return Err(err.into());
                    }
                };
            }
        }

        Ok(())
    }
}