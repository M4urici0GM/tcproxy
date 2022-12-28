use chrono::Utc;
use std::sync::Arc;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use tracing::debug;

use tcproxy_core::transport::TransportReader;
use tcproxy_core::{AsyncCommand};
use tcproxy_core::{Result, TcpFrame};

use crate::{ClientState, ListenArgs, Shutdown};
use crate::commands::{DataPacketCommand, IncomingSocketCommand, RemoteDisconnectedCommand};

pub struct TcpFrameReader {
    sender: Sender<TcpFrame>,
    reader: Box<dyn TransportReader>,
    state: Arc<ClientState>,
    args: Arc<ListenArgs>,
    _shutdown_complete_tx: Sender<()>
}

impl TcpFrameReader {
    pub fn new<T>(
        sender: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
        reader: T,
        args: &Arc<ListenArgs>,
        shutdown_complete_tx: &Sender<()>
    ) -> Self
        where T: TransportReader + 'static {
        Self {
            args: args.clone(),
            sender: sender.clone(),
            state: state.clone(),
            reader: Box::new(reader),
            _shutdown_complete_tx: shutdown_complete_tx.clone()
        }
    }

    pub fn spawn(mut self, shutdown: Shutdown) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let result = TcpFrameReader::start(&mut self, shutdown).await;
            debug!("tcpframe_reader::start finished with {:?}", result);
            Ok(())
        })
    }

    async fn start(&mut self, mut shutdown: Shutdown) -> Result<()> {
        while !shutdown.is_shutdown() {
            let maybe_frame = tokio::select! {
                res = self.reader.next() => res?,
                _ = shutdown.recv() => {
                    debug!("received stop signal from cancellation token");
                    return Ok(())
                }
            };

            let msg = match maybe_frame {
                Some(f) => f,
                None => {
                    debug!("received none from framereader.");
                    break;
                }
            };

            debug!("received new frame from server: {}", msg);
            let mut command: Box<dyn AsyncCommand<Output = Result<()>>> = match msg {
                TcpFrame::DataPacket(data) => Box::new(DataPacketCommand::new(
                    data.connection_id(),
                    data.buffer(),
                    &self.state,
                )),
                TcpFrame::IncomingSocket(data) => Box::new(IncomingSocketCommand::new(
                    data.connection_id(),
                    &self.sender,
                    &self.state,
                    &self.args,
                )),
                TcpFrame::RemoteSocketDisconnected(data) => {
                    Box::new(RemoteDisconnectedCommand::new(data.connection_id(), &self.state))
                }
                TcpFrame::ClientConnectedAck(data) => {
                    debug!("Remote proxy listening in {}:{}", "127.0.0.1", data.port());
                    self.state.update_remote_ip(&data.port().to_string());
                    continue;
                }
                TcpFrame::Pong(_) => {
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
