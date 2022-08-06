use tcproxy_cli::{IncomingSocketCommand, ClientState, RemoteDisconnectedCommand, DataPacketCommand};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};
use tokio::task::JoinHandle;
use tokio::time::{self, Duration, Instant};
use tracing::{debug, error, info};

use tcproxy_core::{Result, TcpFrame, FrameReader, Command};

async fn send_connected_frame(writer: &mut OwnedWriteHalf) {
    match writer
        .write_buf(&mut TcpFrame::ClientConnected.to_buffer())
        .await
    {
        Ok(_) => {
            info!("send initial frame to server..");
        }
        Err(err) => {
            error!("Failed sending initial frame to server: {}", err);
        }
    };
}

fn start_ping(sender: Sender<TcpFrame>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            info!("Waiting for next ping to occur");
            time::sleep_until(Instant::now() + Duration::from_secs(10)).await;
            match sender.send(TcpFrame::Ping).await {
                Ok(_) => {
                    info!("Sent ping frame..");
                }
                Err(err) => {
                    error!("Failed to send ping. aborting. {}", err);
                    break;
                }
            };
        }
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let tcp_connection = match TcpStream::connect("127.0.0.1:8080").await {
        Ok(stream) => {
            debug!("Connected to server..");
            stream
        }
        Err(err) => {
            error!("Failed to connect to server. Check you network connection and try again.");
            return Err(format!("Failed when connecting to server: {}", err).into());
        }
    };

    let state = Arc::new(ClientState::new());
    let (mut reader, mut writer) = tcp_connection.into_split();
    send_connected_frame(&mut writer).await;

    let (main_sender, mut main_receiver) = mpsc::channel::<TcpFrame>(10000);

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = main_receiver.recv().await {
            let _ = writer.write_buf(&mut msg.to_buffer()).await;
            let _ = writer.flush().await;
        }

        info!("Reached end of stream.");
    });

    let ping_task = start_ping(main_sender.clone());
    let foward_task: JoinHandle<Result<()>> = tokio::spawn(async move {
        let mut frame_reader = FrameReader::new(&mut reader);

        loop {
            let maybe_frame = frame_reader.receive_frame().await?;
            let msg = match maybe_frame {
                Some(f) => f,
                None => {
                    debug!("received none from framereader.");
                    break;
                }
            };

            debug!("received new frame from server: {}", msg);
            let command: Box<dyn Command> = match msg {
               
                TcpFrame::DataPacketHost { connection_id, buffer } => {
                    Box::new(DataPacketCommand::new(connection_id, buffer, &state))
                },
                TcpFrame::IncomingSocket { connection_id } => {
                    Box::new(IncomingSocketCommand::new(connection_id, &main_sender, &state))
                },
                TcpFrame::RemoteSocketDisconnected { connection_id } => {
                    Box::new(RemoteDisconnectedCommand::new(connection_id, &state))
                },
                TcpFrame::ClientConnectedAck { port } => {
                    info!("Remote proxy listening in {}:{}", "127.0.0.1", port);
                    continue;
                },
                packet => {
                    debug!("invalid data packet received. {}", packet);
                    continue;
                }
            };

            command
                .handle()
                .await?;
        }

        Ok(())
    });

    tokio::select! {
        _ = receive_task => {
            debug!("Receive task finished.");
        },
 
        res = foward_task => {
            debug!("Forward to server task finished. {:?}", res);
        },
        _ = ping_task => {
            debug!("ping task finished.");
        }
    };

    Ok(())
}
