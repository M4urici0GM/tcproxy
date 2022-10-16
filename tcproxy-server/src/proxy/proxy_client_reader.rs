use tokio::{sync::mpsc::Sender, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use tcproxy_core::transport::{DefaultTransportReader, TransportReader};
use tcproxy_core::{Result, TcpFrame};

use crate::commands::{ClientConnectedCommand, DataPacketClientCommand};
use crate::proxy::FrameHandler;

/// Responsible for reading commands / frames from client and processing them.
pub struct ClientFrameReader {
    frame_tx: Sender<TcpFrame>,
    reader: Box<dyn TransportReader>,
    frame_handler: Box<dyn FrameHandler>,
}

impl ClientFrameReader {
    pub fn new<'a, T, V>(sender: &Sender<TcpFrame>, reader: V, frame_handler: T) -> Self
    where
        T: FrameHandler + 'static,
        V: TransportReader + 'static
    {
        Self {
            reader: Box::new(reader),
            frame_tx: sender.clone(),
            frame_handler: Box::new(frame_handler),
        }
    }

    pub fn start_reading(
        mut self,
        cancellation_token: CancellationToken,
    ) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let read_task = ClientFrameReader::start(&mut self, cancellation_token.child_token());

            tokio::select! {
                res = read_task => {
                    match res {
                        Ok(_) => {
                            debug!("read task has been finished with no errors");
                        },
                        Err(err) => {
                            debug!("read task has been finished with error: {}", err);
                        }
                    }
                },
                _ = cancellation_token.cancelled() => {
                    debug!("task requested to be cancelled");
                }
            }

            Ok(())
        })
    }

    /// Start listening for frames, and handling them.
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
            match self
                .frame_handler
                .handle_frame(frame, cancellation_token.child_token())
                .await?
            {
                Some(frame_result) =>self.frame_tx.send(frame_result).await?,
                None => {}
            }
        }

        Ok(())
    }
}
