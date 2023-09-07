use std::sync::Arc;

use async_trait::async_trait;
use tcproxy_core::{
    framing::{Authenticate, AuthenticateAck, Error, Reason},
    TcpFrame,
};
use tokio::sync::mpsc::Sender;

use super::authenticate;
use crate::{
    commands::{authenticate::authenticate::AuthenticateCommandError, NewFrameHandler},
    ClientState,
};

pub struct AuthenticateFrameHandler(tcproxy_core::framing::Authenticate);

impl From<Authenticate> for AuthenticateFrameHandler {
    fn from(value: Authenticate) -> Self {
        Self(value)
    }
}

impl Into<Box<dyn NewFrameHandler>> for AuthenticateFrameHandler {
    fn into(self) -> Box<dyn NewFrameHandler> {
        Box::new(self)
    }
}

#[async_trait]
impl NewFrameHandler for AuthenticateFrameHandler {
    async fn execute(
        &self,
        tx: &Sender<TcpFrame>,
        state: &Arc<ClientState>,
    ) -> tcproxy_core::Result<Option<TcpFrame>> {
        let auth_manager = state.get_auth_manager();
        if auth_manager.is_authenticated() {
            return Ok(Some(TcpFrame::Error(Error::new(
                &Reason::AuthenticationFailed,
                &[],
            ))));
        }

        let (user, token) = match authenticate::challenge(self.0.grant_type(), state).await {
            Ok(acc_details) => acc_details,
            Err(AuthenticateCommandError::AuthenticationFailed) => {
                return Ok(Some(TcpFrame::Error(Error::new(
                    &Reason::AuthenticationFailed,
                    &[],
                ))));
            }
            Err(AuthenticateCommandError::Other(err)) => {
                tracing::error!("failed when trying to fetch account details: {}", err);
                // At this point, we couldn't handle the error.
                // Send a error response to client, and close the connection.
                tx.send(TcpFrame::Error(Error::new(&Reason::UnexpectedError, &[])))
                    .await?;
                return Err(format!("unexpected error: {:?}", err).into());
            }
        };

        tracing::info!("successfully authenticated, sending AuthenticateAck frame back");
        auth_manager.set_authentication_details(&user);

        Ok(Some(TcpFrame::AuthenticateAck(AuthenticateAck::new(
            &user.id().to_string(),
            user.email(),
            token,
        ))))
    }
}
