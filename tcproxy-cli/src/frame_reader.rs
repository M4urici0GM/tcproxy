use chrono::Utc;
use std::sync::Arc;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use tracing::debug;

use tcproxy_core::transport::TransportReader;
use tcproxy_core::AsyncCommand;
use tcproxy_core::{Result, TcpFrame};

use crate::commands::{DataPacketCommand, IncomingSocketCommand, RemoteDisconnectedCommand};
use crate::{ClientState, ListenArgs, Shutdown};

pub struct TcpFrameReader {
    sender: Sender<TcpFrame>,
    reader: TransportReader,
    state: Arc<ClientState>,
    args: Arc<ListenArgs>,
    _shutdown_complete_tx: Sender<()>,
}

impl TcpFrameReader {
    pub fn new(
        sender: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
        args: &Arc<ListenArgs>,
        reader: TransportReader,
        shutdown_complete_tx: &Sender<()>,
    ) -> Self {
        Self {
            args: args.clone(),
            sender: sender.clone(),
            state: state.clone(),
            reader,
            _shutdown_complete_tx: shutdown_complete_tx.clone(),
        }
    }

    pub fn spawn(mut self, mut shutdown: Shutdown) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
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
                    TcpFrame::SocketConnected(data) => Box::new(IncomingSocketCommand::new(
                        data.connection_id(),
                        &self.sender,
                        &self.state,
                        &self.args,
                    )),
                    TcpFrame::SocketDisconnected(data) => {
                        debug!("remote socket disconnected");
                        Box::new(RemoteDisconnectedCommand::new(
                            data.connection_id(),
                            &self.state,
                        ))
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

            debug!("tcpframe_reader::start finished");
            Ok(())
        })
    }
}
