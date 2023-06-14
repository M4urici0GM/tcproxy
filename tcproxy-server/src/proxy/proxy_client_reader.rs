use tokio::{task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};

use tcproxy_core::transport::TransportReader;
use tcproxy_core::{Result};

use crate::proxy::FrameHandler;

/// Responsible for reading commands / frames from client and processing them.
pub struct ClientFrameReader {
    reader: TransportReader,
    frame_handler: Box<dyn FrameHandler>,
}

impl ClientFrameReader {
    pub fn new<T>(reader: TransportReader, frame_handler: T) -> Self
    where
        T: FrameHandler + 'static,
    {
        Self {
            reader,
            frame_handler: Box::new(frame_handler),
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
            self.frame_handler
                .handle(frame, cancellation_token.child_token())
                .await?;
        }

        Ok(())
    }
}
