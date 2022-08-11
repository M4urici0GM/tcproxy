use async_trait::async_trait;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::{debug, error};

use tcproxy_core::{transport::TcpFrameTransport, Command, Result, TcpFrame};

use crate::{ClientState, ConsoleUpdater, ListenArgs, PingSender, TcpFrameReader, TcpFrameWriter};

pub struct ListenCommand {
    args: Arc<ListenArgs>,
}

impl ListenCommand {
    pub fn new(args: Arc<ListenArgs>) -> Self {
        Self {
            args,
        }
    }

    /// connects to remote server.
    async fn connect(&self) -> Result<TcpStream> {
        match TcpStream::connect("20.197.199.20:8080").await {
            Ok(stream) => {
                debug!("Connected to server..");
                Ok(stream)
            }
            Err(err) => {
                println!("{} {}", 124, 123);

                error!("Failed to connect to server. Check you network connection and try again.");
                return Err(format!("Failed when connecting to server: {}", err).into());
            }
        }
    }
}

#[async_trait]
impl Command for ListenCommand {
    type Output = ();

    async fn handle(&mut self) -> Result<()> {
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
}
