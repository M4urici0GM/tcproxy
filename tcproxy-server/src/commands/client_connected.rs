use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use tracing::debug;

use tcproxy_core::framing::ClientConnectedAck;
use tcproxy_core::{AsyncCommand, Result, TcpFrame};

pub struct ClientConnectedCommand {
    client_sender: Sender<TcpFrame>,
}

impl ClientConnectedCommand {
    pub fn new(sender: &Sender<TcpFrame>) -> Self {
        Self {
            client_sender: sender.clone(),
        }
    }

    pub fn boxed_new(sender: &Sender<TcpFrame>) -> Box<Self> {
        let local_self = ClientConnectedCommand::new(&sender);
        Box::new(local_self)
    }
}

#[async_trait]
impl AsyncCommand for ClientConnectedCommand {
    type Output = Result<()>;

    async fn handle(&mut self) -> Self::Output {
        debug!("received connection client command");
        self.client_sender
            .send(TcpFrame::ClientConnectedAck(ClientConnectedAck::new()))
            .await?;

        Ok(())
    }
}
