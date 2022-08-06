use std::sync::Arc;
use clap::Parser;
use tokio::sync::mpsc::{self, Receiver};
use tokio::task::JoinHandle;
use tokio::{net::TcpStream, sync::mpsc::Sender};
use emoji_printer::print_emojis;
use tracing::{debug, error};

use tcproxy_cli::{ClientArgs, ClientState, PingSender, TcpFrameReader, TcpFrameWriter};
use tcproxy_core::{Result, TcpFrame, TcpFrameTransport};

struct App {
    args: ClientArgs,
}

struct ConsoleUpdater {
    receiver: Receiver<i32>,
    state: Arc<ClientState>,
}

impl ConsoleUpdater {
    pub fn new(receiver: Receiver<i32>, state: &Arc<ClientState>) -> Self {
        Self {
            receiver,
            state: state.clone(),
        }
    }

    pub fn spawn(mut self) -> JoinHandle<Result<()>> {
        tokio::spawn(async move {
            let _ = ConsoleUpdater::start(&mut self).await;
            Ok(())
        })
    }

    fn clear(&self) {
        print!("\x1B[2J\x1B[1;1H");
    }


    fn print_state(&self) {
        self.clear();
        let state = self.state.get_console_status();
        let msg = print_emojis(&format!(":rocket: Server running at {}\n:dizzy: Ping: {:.2}ms\n:anchor: Connections: {}", state.remote_ip, state.ping, state.connections));
        println!("{}", msg);
    }

    async fn start(&mut self) {
        self.print_state();
        while let Some(_) = self.receiver.recv().await {
            self.print_state();
        }

        self.clear();
    }
}

impl App {
    pub fn new(args: ClientArgs) -> Self {
        Self {
            args,
        }
    }

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
     

        let console_task = ConsoleUpdater::new(console_receiver, &state).spawn();
        let receive_task = TcpFrameWriter::new(receiver, writer).spawn();
        let ping_task = PingSender::new(&sender, &state, Some(5)).spawn();
        let foward_task = TcpFrameReader::new(&sender, &state, &console_sender, reader).spawn();

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

    pub async fn connect(&self) -> Result<TcpStream> {
        match TcpStream::connect("127.0.0.1:8080").await {
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = ClientArgs::parse();
    App::new(args)
        .start()
        .await?;

    Ok(())
}
