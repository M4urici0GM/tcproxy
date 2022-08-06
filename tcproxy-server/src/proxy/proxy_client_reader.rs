use std::net::Ipv4Addr;
use std::sync::Arc;

use tcproxy_core::{TcpFrame, Result, Command, TransportReader};
use tokio::{task::JoinHandle, sync::mpsc::Sender};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use crate::ProxyState;
use crate::commands::{DataPacketClientCommand, ClientConnectedCommand};
use crate::commands::{PingCommand, LocalClientDisconnectedCommand};


pub struct ProxyClientStreamReader {
    pub(crate) target_ip: Ipv4Addr,
    pub(crate) sender: Sender<TcpFrame>,
    pub(crate) state: Arc<ProxyState>,
    pub(crate) reader: TransportReader,
}

impl ProxyClientStreamReader {
    pub fn start_reading(mut self, cancellation_token: CancellationToken) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let _ = ProxyClientStreamReader::start(&mut self, cancellation_token).await;
            Ok(())
        })
    }

    async fn start(&mut self, cancellation_token: CancellationToken) -> Result<()> {
        loop {
            let maybe_frame = self.reader.next().await?;
            let frame = match maybe_frame {
                Some(f) => f,
                None => {
                    info!("received none from frame reader");
                    break;
                }
            };

            debug!("received new frame from client {}", frame);
            let mut command_handler: Box<dyn Command> = match frame {
                TcpFrame::Ping => Box::new(PingCommand::new(&self.sender)),
                TcpFrame::LocalClientDisconnected { connection_id } => {
                    Box::new(LocalClientDisconnectedCommand::new(connection_id, &self.state))
                }
                TcpFrame::DataPacketClient { connection_id, buffer, buffer_size: _ } => {
                    Box::new(DataPacketClientCommand::new(buffer, &connection_id, &&self.state))
                },
                TcpFrame::ClientConnected => {
                    Box::new(ClientConnectedCommand {
                        target_ip: self.target_ip.clone(),
                        state: self.state.clone(),
                        sender: self.sender.clone(),
                        cancellation_token: cancellation_token.child_token(),
                    })
                },
                _ => {
                    debug!("invalid frame received.");
                    continue;
                }
            };

            command_handler.handle().await?;
        }

        Ok(())
    }
}
