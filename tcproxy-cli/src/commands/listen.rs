use std::net::SocketAddr;
use std::sync::Arc;
use std::str::FromStr;
use async_trait::async_trait;
use tokio::net::TcpStream as TokioTcpStream;
use tokio::sync::mpsc;
use tracing::{debug, error};


use tcproxy_core::{transport::TcpFrameTransport, AsyncCommand, Result, TcpFrame};
use tcproxy_core::tcp::TcpStream;

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
        let addr = SocketAddr::from_str("127.0.0.1:8080")?;
        match TokioTcpStream::connect(addr).await {
            Ok(stream) => {
                debug!("Connected to server..");
                let socket_addr = stream.peer_addr().unwrap();
                Ok(TcpStream::new(stream, socket_addr))
            }
            Err(err) => {
                println!("{} {}", 124, 123);

                error!("Failed to connect to server. Check you network connection and try again.");
                Err(format!("Failed when connecting to server: {}", err).into())
            }
        }
    }
}

#[async_trait]
impl AsyncCommand for ListenCommand {
    type Output = Result<()>;

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
