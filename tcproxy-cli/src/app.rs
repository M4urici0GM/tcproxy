use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::{debug, error};

use tcproxy_core::transport::TcpFrameTransport;
use tcproxy_core::Result;
use tcproxy_core::TcpFrame;

use crate::ConsoleUpdater;
use crate::ListenArgs;
use crate::PingSender;
use crate::TcpFrameReader;
use crate::TcpFrameWriter;
use crate::ClientState;

/// represents main app logic.
pub struct App {
    args: Arc<ListenArgs>,
}

impl App {
    pub fn new(args: ListenArgs) -> Self {
        Self {
            args: Arc::new(args),
        }
    }

    /// does initial handshake and start listening for remote connections.
    pub async fn start(&self) -> Result<()> {
        if self.args.is_debug() {
            tracing_subscriber::fmt::init();
        }

        let connection = self.connect().await?;
        let (console_sender, console_receiver) = mpsc::channel::<i32>(10);
        let (sender, receiver) = mpsc::channel::<TcpFrame>(10000);
        let (reader, mut writer) = TcpFrameTransport::new(connection).split();

        let state = Arc::new(ClientState::new(&console_sender));

        writer.send(TcpFrame::ClientConnected).await?;
        writer.send(TcpFrame::Ping).await?;

        let ping_task = PingSender::new(&sender, &state, self.args.ping_interval()).spawn();
        let console_task = ConsoleUpdater::new(console_receiver, &state, &self.args).spawn();
        let receive_task = TcpFrameWriter::new(receiver, writer).spawn();
        let foward_task = TcpFrameReader::new(&sender, &state, reader, &self.args).spawn();

        tokio::select! {
            res = console_task => {
                println!("{:?}", res);
                debug!("console task finished.");
            }
            _ = receive_task => {
                debug!("receive task finished.");
            },

            res = foward_task => {
                debug!("forward to server task finished. {:?}", res);
            },
            _ = ping_task => {
                debug!("ping task finished.");
            }
        };

        Ok(())
    }

    /// connects to remote server.
    async fn connect(&self) -> Result<TcpStream> {
        match TcpStream::connect("192.168.0.221:8080").await {
            Ok(stream) => {
                debug!("Connected to server..");
                Ok(stream)
            }
            Err(err) => {
                error!("Failed to connect to server. Check you network connection and try again.");
                return Err(format!("Failed when connecting to server: {}", err).into());
            }
        }
    }
}
