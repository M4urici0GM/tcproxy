use std::sync::Arc;
use chrono::Utc;
use tracing::{debug, info};
use tcproxy_core::{TcpFrame, TransportReader, Result, Command};
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use crate::{ClientState, DataPacketCommand, IncomingSocketCommand, RemoteDisconnectedCommand};

pub struct TcpFrameReader {
  sender: Sender<TcpFrame>,
  reader: TransportReader,
  state: Arc<ClientState>,
  console_sender: Sender<i32>
}

impl TcpFrameReader {
  pub fn new(sender: &Sender<TcpFrame>, state: &Arc<ClientState>, console_sender: &Sender<i32>, reader: TransportReader) -> Self {
      Self {
          sender: sender.clone(),
          state: state.clone(),
          reader,
          console_sender: console_sender.clone(),
      }
  }

  pub fn spawn(mut self) -> JoinHandle<Result<()>> {
      tokio::spawn(async move {
          let result = TcpFrameReader::start(&mut self).await;
          debug!("tcpframe_reader::start finished with {:?}", result);
          Ok(())
      })
  }

  async fn start(&mut self) -> Result<()> {
      loop {
          let maybe_frame = self.reader.next().await?;
          let msg = match maybe_frame {
              Some(f) => f,
              None => {
                  debug!("received none from framereader.");
                  break;
              }
          };

          debug!("received new frame from server: {}", msg);
          let command: Box<dyn Command> = match msg {
                  TcpFrame::DataPacketHost { connection_id, buffer} => {
                    Box::new(DataPacketCommand::new(connection_id, buffer, &self.state))
                  },
                  TcpFrame::IncomingSocket { connection_id } => Box::new(
                      IncomingSocketCommand::new(connection_id, &self.sender, &self.state),
                  ),
                  TcpFrame::RemoteSocketDisconnected { connection_id } => {
                      Box::new(RemoteDisconnectedCommand::new(connection_id, &self.state))
                  }
                  TcpFrame::ClientConnectedAck { port } => {
                      debug!("Remote proxy listening in {}:{}", "127.0.0.1", port);
                      self.state.update_remote_ip(&port.to_string());
                      continue;
                  },
                  TcpFrame::Pong => {
                    let time = Utc::now();
                    self.state.update_last_ping(time);

                    continue;
                  },
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
