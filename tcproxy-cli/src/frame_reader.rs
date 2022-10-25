use chrono::Utc;
use std::sync::Arc;
use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tracing::debug;

use tcproxy_core::transport::TransportReader;
use tcproxy_core::AsyncCommand;
use tcproxy_core::{transport::DefaultTransportReader, Result, TcpFrame};

use crate::{
    ClientState, DataPacketCommand, IncomingSocketCommand, ListenArgs, RemoteDisconnectedCommand,
};

pub struct TcpFrameReader {
    sender: Sender<TcpFrame>,
    reader: DefaultTransportReader,
    state: Arc<ClientState>,
    args: Arc<ListenArgs>,
}

impl TcpFrameReader {
    pub fn new(
        sender: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
        reader: DefaultTransportReader,
        args: &Arc<ListenArgs>,
    ) -> Self {
        Self {
            args: args.clone(),
            sender: sender.clone(),
            state: state.clone(),
            reader,
        }
    }

    pub fn spawn(mut self) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let result = TcpFrameReader::start(&mut self).await;
            debug!("tcpframe_reader::start finished with {:?}", result);
            Ok(())
        })
    }

    async fn start(&mut self) -> Result<()> {
        loop {
            let maybe_frame = self.reader.next().await?;
            let msg = match maybe_frame {
                Some(f) => f,
                None => {
                    debug!("received none from framereader.");
                    break;
                }
            };

            debug!("received new frame from server: {}", msg);
            let mut command: Box<dyn AsyncCommand<Output = Result<()>>> = match msg {
                TcpFrame::HostPacket(data) => Box::new(DataPacketCommand::new(
                    data.connection_id(),
                    data.buffer(),
                    data.buffer_size(),
                    &self.state,
                )),
                TcpFrame::IncomingSocket { connection_id } => Box::new(IncomingSocketCommand::new(
                    connection_id,
                    &self.sender,
                    &self.state,
                    &self.args,
                )),
                TcpFrame::RemoteSocketDisconnected { connection_id } => {
                    Box::new(RemoteDisconnectedCommand::new(connection_id, &self.state))
                }
                TcpFrame::ClientConnectedAck { port } => {
                    debug!("Remote proxy listening in {}:{}", "127.0.0.1", port);
                    self.state.update_remote_ip(&port.to_string());
                    continue;
                }
                TcpFrame::Pong => {
                    let time = Utc::now();
                    self.state.update_last_ping(time);

                    continue;
                }
                packet => {
                    debug!("invalid data packet received. {}", packet);
                    continue;
                }
            };

            command.handle().await?;
        }

        Ok(())
    }
}
