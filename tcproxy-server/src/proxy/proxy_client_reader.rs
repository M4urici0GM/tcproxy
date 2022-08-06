use std::net::Ipv4Addr;
use std::sync::Arc;

use tcproxy_core::{TcpFrame, Result, FrameReader, Command};
use tokio::{task::JoinHandle, sync::mpsc::Sender};
use tokio::net::tcp::OwnedReadHalf;
use tokio_util::sync::CancellationToken;
use tracing::debug;

use crate::ProxyState;
use crate::commands::{DataPacketClientCommand, ClientConnectedCommand};
use crate::commands::{PingCommand, LocalClientDisconnectedCommand};


pub struct ProxyClientStreamReader {
    pub(crate) target_ip: Ipv4Addr,
    pub(crate) sender: Sender<TcpFrame>,
    pub(crate) state: Arc<ProxyState>,
}

impl ProxyClientStreamReader {
    pub fn start_reading(&mut self, mut reader: OwnedReadHalf, cancellation_token: CancellationToken) -> JoinHandle<Result<()>> {
        let target_ip = self.target_ip.clone();
        let sender = self.sender.clone();
        let proxy_state = self.state.clone();

        tokio::spawn(async move {
            let mut frame_reader = FrameReader::new(&mut reader);
            loop {
                let maybe_frame = frame_reader.receive_frame().await?;
                let frame = match maybe_frame {
                    Some(f) => f,
                    None => {
                        break;
                    }
                };

                let command_handler: Box<dyn Command> = match frame {
                    TcpFrame::Ping => Box::new(PingCommand::new(&sender)),
                    TcpFrame::LocalClientDisconnected { connection_id } => {
                        Box::new(LocalClientDisconnectedCommand::new(connection_id, proxy_state.clone()))
                    }
                    TcpFrame::DataPacketClient { connection_id, buffer} => {
                        Box::new(DataPacketClientCommand::new(buffer, &connection_id, &proxy_state))
                    },
                    TcpFrame::ClientConnected => {
                        Box::new(ClientConnectedCommand {
                            target_ip: target_ip.clone(),
                            state: proxy_state.clone(),
                            sender: sender.clone(),
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
        })
    }
}
