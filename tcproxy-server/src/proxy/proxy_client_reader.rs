use std::sync::Arc;

use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use tcproxy_core::transport::TransportReader;
use tcproxy_core::{Result, TcpFrame};

use crate::ClientState;
use crate::commands::authenticate::AuthenticateFrameHandler;
use crate::commands::{
    ClientConnectedHandler, DataPacketHandler, NewFrameHandler, PingFrameHandler,
    SocketDisconnectedHandler,
};

/// Responsible for reading commands / frames from client and processing them.
pub struct ClientFrameReader {
    reader: TransportReader,
    sender: Sender<TcpFrame>,
    state: Arc<ClientState>,
}

impl ClientFrameReader {

    pub fn new(reader: TransportReader, state: &Arc<ClientState>, tx: &Sender<TcpFrame>) -> Self {
        Self {
            reader,
            state: state.clone(),
            sender: tx.clone(),
        }
    }
    
    pub fn spawn(self, cancellation_token: CancellationToken) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let _ = self.start(cancellation_token.child_token()).await;
            Ok(())
        })
    }

    /// Start listening for frames, and handling them.
    async fn start(mut self, cancellation_token: CancellationToken) -> Result<()> {
        while !cancellation_token.is_cancelled() {
            let maybe_frame = self.reader.next().await?;
            let frame = match maybe_frame {
                Some(f) => f,
                None => {
                    info!("received none from frame reader");
                    break;
                }
            };

            debug!("received new frame from client {}", frame);
            handle_frame(frame, &self.sender, &self.state).await?;
        }

        Ok(())
    }
}

async fn handle_frame(frame: TcpFrame, sender: &Sender<TcpFrame>, state: &Arc<ClientState>) -> Result<()> {
    use TcpFrame as F;
    let command_handler: Box<dyn NewFrameHandler> = match frame {
        F::Ping(data) => PingFrameHandler::from(data).into(),
        F::DataPacket(data) => DataPacketHandler::from(data).into(),
        F::Authenticate(data) => AuthenticateFrameHandler::from(data).into(),
        F::ClientConnected(data) => ClientConnectedHandler::from(data).into(),
        F::SocketDisconnected(data) => SocketDisconnectedHandler::from(data).into(),
        _ => {
            debug!("invalid frame received.");
            return Ok(());
        }
    };

    match command_handler.execute(&sender, &state).await? {
        None => Ok(()),
        Some(frame) => {
            sender.send(frame.clone()).await?;
            Ok(())
        }
    }
}
